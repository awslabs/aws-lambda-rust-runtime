#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The official Rust runtime for AWS Lambda.
//!
//! There are two mechanisms available for defining a Lambda function:
//! 1. The `lambda` attribute maco, which generates the boilerplate to
//!    to launch and run a Lambda function.
//!
//!    The [`#[lambda]`] attribute _must_ be placed on an asynchronous main function.
//!    However, as asynchronous main functions are not legal valid Rust
//!    this means that the main function must also be decorated using a
//!    [`#[tokio::main]`] attribute macro. This is available from
//!    the [Tokio] crate.
//!
//! 2. A type that conforms to the [`Handler`] trait. This type can then be passed
//!    to the the `lambda::run` function, which launches and runs the Lambda runtime.
//!
//! An asynchronous function annotated with the `#[lambda]` attribute must
//! accept an argument of type `A` which implements [`serde::Deserialize`], a [`lambda::Context`] and
//! return a `Result<B, E>`, where `B` implements [`serde::Serializable`]. `E` is
//! any type that implements `Into<Box<dyn std::error::Error + Send + Sync + 'static>>`.
//!
//! ```no_run
//! use lambda::{lambda, Context};
//! use serde_json::Value;
//!
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[lambda]
//! #[tokio::main]
//! async fn main(event: Value, _: Context) -> Result<Value, Error> {
//!     Ok(event)
//! }
//! ```
//!
//! [`Handler`]: trait.Handler.html
//! [`lambda::Context`]: struct.Context.html
//! [`lambda`]: attr.lambda.html
//! [`#[tokio::main]`]: https://docs.rs/tokio/0.2.21/tokio/attr.main.html
//! [Tokio]: https://docs.rs/tokio/
pub use crate::types::Context;
use client::Client;
pub use lambda_attributes::lambda;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    env, fmt,
    future::Future,
    sync::Arc,
};
use tokio::stream::{Stream, StreamExt};

mod client;
mod requests;
#[cfg(test)]
mod simulated;
/// Types available to a Lambda function.
mod types;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest};
use types::Diagnostic;

/// Error type that lambdas may result in
pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Configuration derived from environment variables.
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
    pub fn from_env() -> Result<Self, Error> {
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

/// A trait describing an asynchronous function `A` to `B`.
pub trait Handler<A, B> {
    /// Errors returned by this handler.
    type Error;
    /// Response of this handler.
    type Fut: Future<Output = Result<B, Self::Error>>;
    /// Handle the incoming event.
    fn call(&self, event: A, context: Context) -> Self::Fut;
}

/// Returns a new [`HandlerFn`] with the given closure.
///
/// [`HandlerFn`]: struct.HandlerFn.html
pub fn handler_fn<F>(f: F) -> HandlerFn<F> {
    HandlerFn { f }
}

/// A [`Handler`] implemented by a closure.
///
/// [`Handler`]: trait.Handler.html
#[derive(Clone, Debug)]
pub struct HandlerFn<F> {
    f: F,
}

impl<F, A, B, Error, Fut> Handler<A, B> for HandlerFn<F>
where
    F: Fn(A, Context) -> Fut,
    Fut: Future<Output = Result<B, Error>> + Send + Sync,
    Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + fmt::Debug,
{
    type Error = Error;
    type Fut = Fut;
    fn call(&self, req: A, ctx: Context) -> Self::Fut {
        (self.f)(req, ctx)
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// # Example
/// ```no_run
/// use lambda::{handler_fn, Context};
/// use serde_json::Value;
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
/// async fn func(event: Value, _: Context) -> Result<Value, Error> {
///     Ok(event)
/// }
/// ```
pub async fn run<A, B, F>(handler: F) -> Result<(), Error>
where
    F: Handler<A, B> + Send + Sync + 'static,
    <F as Handler<A, B>>::Fut: Future<Output = Result<B, <F as Handler<A, B>>::Error>> + Send + Sync + 'static,
    <F as Handler<A, B>>::Error: fmt::Debug + Send + Sync + 'static,
    A: for<'de> Deserialize<'de> + Send + Sync + 'static,
    B: Serialize + Send + Sync + 'static,
{
    let config = Config::from_env()?;
    let uri = config.endpoint.try_into().expect("Unable to convert to URL");
    let client = Client::with(uri, hyper::Client::new());
    let incoming = incoming(&client);
    run_inner(&client, incoming, handler).await?;

    Ok(())
}

fn incoming(client: &Client) -> impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + '_ {
    async_stream::stream! {
        loop {
            let req = NextEventRequest.into_req().expect("Unable to construct request");
            let res = client.call(req).await;
            yield res;
        }
    }
}

async fn run_inner<A, B, F>(
    client: &Client,
    incoming: impl Stream<Item = Result<http::Response<hyper::Body>, Error>>,
    handler: F,
) -> Result<(), Error>
where
    F: Handler<A, B> + Send + Sync + 'static,
    <F as Handler<A, B>>::Fut: Future<Output = Result<B, <F as Handler<A, B>>::Error>> + Send + Sync + 'static,
    <F as Handler<A, B>>::Error: fmt::Debug + Send + Sync + 'static,
    A: for<'de> Deserialize<'de> + Send + Sync + 'static,
    B: Serialize + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    tokio::pin!(incoming);
    while let Some(event) = incoming.next().await {
        let event = event?;
        let (parts, body) = event.into_parts();

        let ctx: Context = Context::try_from(parts.headers)?;
        let body = hyper::body::to_bytes(body).await?;
        let body = serde_json::from_slice(&body)?;

        let handler = Arc::clone(&handler);
        let request_id = &ctx.request_id.clone();
        let task = tokio::spawn(async move { handler.call(body, ctx) });

        let req = match task.await {
            Ok(response) => match response.await {
                Ok(response) => EventCompletionRequest {
                    request_id,
                    body: response,
                }
                .into_req(),
                Err(err) => EventErrorRequest {
                    request_id,
                    diagnostic: Diagnostic {
                        error_type: type_name_of_val(&err).to_owned(),
                        error_message: format!("{:?}", err),
                    },
                }
                .into_req(),
            },
            Err(err) if err.is_panic() => EventErrorRequest {
                request_id,
                diagnostic: Diagnostic {
                    error_type: type_name_of_val(&err).to_owned(),
                    error_message: "Lambda panicked!".to_owned(),
                },
            }
            .into_req(),
            Err(_) => unreachable!("tokio::task should not be canceled"),
        };
        let req = req?;
        client.call(req).await.expect("Unable to send response to Runtime APIs");
    }

    Ok(())
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}
