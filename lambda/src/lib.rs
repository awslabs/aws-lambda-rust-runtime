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
//! ```no_run
//! use lambda::lambda;
//! use serde_json::Value;
//!
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[lambda]
//! #[tokio::main]
//! async fn main(event: Value) -> Result<Value, Error> {
//!     Ok(event)
//! }
//! ```
pub use crate::types::LambdaCtx;
use anyhow::Error;
use client::Client;
use futures::stream::{Stream, StreamExt};
use genawaiter::{sync::gen, yield_};
pub use lambda_attributes::lambda;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    env, fmt,
    future::Future,
};

mod client;
mod requests;
mod simulated;
/// Types availible to a Lambda function.
mod types;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest};
use types::Diagnostic;

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

tokio::task_local! {
    pub static INVOCATION_CTX: types::LambdaCtx;
}

/// A trait describing an asynchronous function `A` to `B.
pub trait Handler<A, B> {
    /// Errors returned by this handler.
    type Error;
    /// The future response value of this handler.
    type Fut: Future<Output = Result<B, Self::Error>>;
    /// Process the incoming event and return the response asynchronously.
    fn call(&mut self, event: A) -> Self::Fut;
}

/// Returns a new `HandlerFn` with the given closure.
pub fn handler_fn<F>(f: F) -> HandlerFn<F> {
    HandlerFn { f }
}

/// A `Handler` or `HttpHandler` implemented by a closure.
#[derive(Clone, Debug)]
pub struct HandlerFn<F> {
    f: F,
}

impl<F, A, B, Error, Fut> Handler<A, B> for HandlerFn<F>
where
    F: Fn(A) -> Fut,
    Fut: Future<Output = Result<B, Error>> + Send,
    Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + fmt::Debug,
{
    type Error = Error;
    type Fut = Fut;
    fn call(&mut self, req: A) -> Self::Fut {
        // we pass along the context here
        (self.f)(req)
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// # Example
/// ```no_run
/// use lambda::handler_fn;
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
/// async fn func(event: Value) -> Result<Value, Error> {
///     Ok(event)
/// }
/// ```
pub async fn run<A, B, F>(handler: F) -> Result<(), Error>
where
    F: Handler<A, B>,
    <F as Handler<A, B>>::Error: fmt::Debug,
    A: for<'de> Deserialize<'de>,
    B: Serialize,
{
    let mut handler = handler;
    let config = Config::from_env()?;
    let uri = config.endpoint.try_into().expect("Unable to convert to URL");
    let client = Client::with(uri, hyper::Client::new());
    let incoming = incoming(&client);
    run_inner(&client, incoming, &mut handler).await?;

    Ok(())
}

/// Runs the lambda function almost entirely in-memory. This is meant for testing.
pub async fn run_simulated<A, B, F>(handler: F, url: &str) -> Result<(), Error>
where
    F: Handler<A, B>,
    <F as Handler<A, B>>::Error: fmt::Debug,
    A: for<'de> Deserialize<'de>,
    B: Serialize,
{
    let mut handler = handler;
    let uri = url.try_into().expect("Unable to convert to URL");
    let client = Client::with(uri, hyper::Client::new());
    let incoming = incoming(&client).take(1);
    run_inner(&client, incoming, &mut handler).await?;

    Ok(())
}

fn incoming(client: &Client) -> impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + '_ {
    gen!({
        loop {
            let req = NextEventRequest.into_req().expect("Unable to construct request");
            yield_!(client.call(req).await)
        }
    })
}

async fn run_inner<A, B, F>(
    client: &Client,
    incoming: impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Unpin,
    handler: &mut F,
) -> Result<(), Error>
where
    F: Handler<A, B>,
    <F as Handler<A, B>>::Error: fmt::Debug,
    A: for<'de> Deserialize<'de>,
    B: Serialize,
{
    let mut incoming = incoming;
    while let Some(event) = incoming.next().await {
        let event = event?;
        let (parts, body) = event.into_parts();

        let ctx: LambdaCtx = LambdaCtx::try_from(parts.headers)?;
        let body = hyper::body::to_bytes(body).await?;
        let body = serde_json::from_slice(&body)?;

        let request_id = &ctx.request_id.clone();
        // handler, B need to be Send
        // let f = tokio::spawn(INVOCATION_CTX.scope(ctx, async move { handler.call(body).await }));
        let f = tokio::spawn(async { handler.call(body).await });

        let req = match f.await {
            Ok(res) => match res {
                Ok(r) => EventCompletionRequest { request_id, body: r }.into_req()?,
                Err(e) => EventErrorRequest {
                    request_id,
                    diagnostic: Diagnostic {
                        error_message: format!("{:?}", e),
                        error_type: type_name_of_val(e).to_owned(),
                    },
                }
                .into_req()?,
            },
            Err(panic) => EventErrorRequest {
                request_id,
                diagnostic: Diagnostic {
                    error_message: "Unexpected panic".to_string(),
                    error_type: "Panic".to_string(),
                },
            }
            .into_req()?,
        };
        client.call(req).await?;
    }

    Ok(())
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}
