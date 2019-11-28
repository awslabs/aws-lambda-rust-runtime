#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The official Rust runtime for AWS Lambda.
//!
//! There are two mechanisms of defining a Lambda function:
//! 1. The `#[lambda]` attribute, which generates the boilerplate needed to
//!    to launch and run a Lambda function. The `#[lambda]` attribute _must_
//!    be placed on an asynchronous main funtion. However, asynchronous main
//!    funtions are not legal valid Rust, which means that a crate like
//!    [Runtime](https://github.com/rustasync/runtime) must be used. A main function
//!    decorated using `#[lamdba]`
//! 2. A type that conforms to the [`Handler`] trait. This type can then be passed
//!    to the the `lambda::run` function, which launches and runs the Lambda runtime.
//!
//! An asynchronous function annotated with the `#[lambda]` attribute must
//! accept an argument of type `A` which implements [`serde::Deserialize`] and
//! return a `Result<B, E>`, where `B` implements [`serde::Serializable`]. `E` is
//! any type that implements `Into<Box<dyn std::error::Error + Send + Sync + 'static>>`.
//!
//! Optionally, the `#[lambda]` annotated function can accept an argument
//! of [`lambda::LambdaCtx`].
//!
//! ```rust
//! use lambda::lambda;
//!
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[lambda]
//! #[tokio::main]
//! async fn main(event: String) -> Result<String, Error> {
//!     Ok(event)
//! }
//! ```
pub use crate::types::LambdaCtx;
use client::{events, Client};
use futures::{pin_mut, prelude::*};
use http::{Request, Response, Uri};
pub use lambda_attributes::lambda;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{convert::TryFrom, env, fmt};
use thiserror::Error;

mod client;
mod requests;
/// Types availible to a Lambda function.
mod types;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest};
use types::Diagnostic;

#[derive(Error, Debug)]
enum Error {
    #[error("error making an http request")]
    Hyper(#[source] hyper::error::Error),
    #[error("invalid URI: {uri}")]
    InvalidUri {
        uri: String,
        #[source]
        source: http::uri::InvalidUri,
    },
    #[error("serialization error")]
    Json {
        #[source]
        source: serde_json::error::Error,
    },
}

/// A struct containing configuration values derived from environment variables.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Config {
    /// The host and port of the [runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
    pub endpoint: String,
    /// The name of the function.
    pub function_name: String,
    /// The amount of memory available to the function in MB.
    pub memory: i32,
    /// The version of the function being executed.
    pub version: String,
    /// The name of the Amazon CloudWatch Logs stream for the function.
    pub log_stream: String,
    /// The name of the Amazon CloudWatch Logs group for the function.
    pub log_group: String,
}

impl Config {
    /// Attempts to read configuration from environment variables.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let conf = Config {
            endpoint: env::var("AWS_LAMBDA_RUNTIME_API")?,
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME")?,
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")?.parse::<i32>()?,
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION")?,
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME")?,
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME")?,
        };
        Ok(conf)
    }
}

/// A trait describing an asynchronous function from `A` to `R`. `A` and `R` must implement [`Deserialize`](serde::Deserialize) and [`Serialize`](serde::Serialize).
pub trait Handler<A, R>
where
    A: for<'de> Deserialize<'de>,
    R: Serialize,
{
    /// Errors returned by this handler.
    type Err;
    /// The future response value of this handler.
    type Fut: Future<Output = Result<R, Self::Err>>;
    /// Process the incoming event and return the response asynchronously.
    ///
    /// # Arguments
    /// * `event` - The data received in the invocation request
    /// * `ctx` - The context for the current invocation
    fn call(&mut self, event: A, ctx: Option<LambdaCtx>) -> Self::Fut;
}

/// A trait describing an asynchronous function from `Request<A>` to `Response<B>`. `A` and `B` must implement [`Deserialize`](serde::Deserialize) and [`Serialize`](serde::Serialize).
pub trait HttpHandler<A, B>: Handler<Request<A>, Response<B>>
where
    Request<A>: for<'de> Deserialize<'de>,
    Response<B>: Serialize,
{
    /// Process the incoming request and return the response asynchronously.
    fn call_http(&mut self, event: Request<A>) -> <Self as Handler<Request<A>, Response<B>>>::Fut;
}

/// Returns a new `HandlerFn` with the given closure.
pub fn handler_fn<Function>(f: Function) -> HandlerFn<Function> {
    HandlerFn { f }
}

/// A `Handler` or `HttpHandler` implemented by a closure.
#[derive(Clone, Debug)]
pub struct HandlerFn<Function> {
    f: Function,
}

impl<Function, Event, Output, Err, Fut> Handler<Event, Output> for HandlerFn<Function>
where
    Function: Fn(Event, Option<LambdaCtx>) -> Fut,
    Event: for<'de> Deserialize<'de>,
    Output: Serialize,
    Fut: Future<Output = Result<Output, Err>> + Send,
    Err: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + fmt::Debug,
{
    type Err = Err;
    type Fut = Fut;
    fn call(&mut self, req: Event, ctx: Option<LambdaCtx>) -> Self::Fut {
        // we pass along the context here
        (self.f)(req, ctx)
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// # Arguments
/// * `handler` - A function or closure that conforms to the `Handler` trait
///
/// # Example
/// ```rust
///
/// use lambda::{handler_fn, LambdaCtx};
///
/// type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = handler_fn(func);
///     lambda::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: String, _ctx: Option<LambdaCtx>) -> Result<String, Error> {
///     Ok(event)
/// }
/// ```
pub async fn run<Function, Event, Output>(mut handler: Function) -> Result<(), anyhow::Error>
where
    Function: Handler<Event, Output>,
    Event: for<'de> Deserialize<'de>,
    Output: Serialize,
    <Function as Handler<Event, Output>>::Err: std::fmt::Debug,
{
    let uri = env::var("AWS_LAMBDA_RUNTIME_API").expect("Unable to find environment variable `AWS_LAMBDA_RUNTIME_API`");
    let uri = Uri::from_str(&uri).map_err(|e| Error::InvalidUri { uri, source: e })?;
    let mut client = Client::new(uri);
    let stream = events(client.clone());
    pin_mut!(stream);

    while let Some(event) = stream.next().await {
        let (parts, body) = event?.into_parts();
        let mut ctx: LambdaCtx = LambdaCtx::try_from(parts.headers)?;
        ctx.env_config = Config::from_env()?;
        let body = body.try_concat().await.map_err(Error::Hyper)?;
        let body = serde_json::from_slice(&body).map_err(|e| Error::Json { source: e })?;

        match handler.call(body, Some(ctx.clone())).await {
            Ok(res) => {
                let body = serde_json::to_vec(&res).map_err(|e| Error::Json { source: e })?;
                let req = EventCompletionRequest {
                    request_id: &ctx.id,
                    body,
                };

                let req = req.into_req()?;
                client.call(req).await?;
            }
            Err(err) => {
                let diagnositic = Diagnostic {
                    error_message: format!("{:?}", err),
                    error_type: type_name_of_val(err).to_string()
                };
                let body = serde_json::to_vec(&diagnositic).map_err(|e| Error::Json { source: e })?;
                let req = EventErrorRequest {
                    request_id: &ctx.id,
                    body
                };

                let req = req.into_req()?;
                client.call(req).await?;
            }
        }
    }

    Ok(())
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}