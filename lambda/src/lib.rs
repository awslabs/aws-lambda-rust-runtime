#![feature(async_await)]
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
//! return a `Result<B, E>`, where `B` implements [serde::Serializable]. `E` is
//! any type that implements `Into<Box<dyn std::error::Error + Send + Sync + 'static>>`.
//!
//! Optionally, the `#[lambda]` annotated function can accept an argument
//! of [`lambda::LambdaCtx`].
//!
//! ```rust
//! #![feature(async_await)]
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
use bytes::Bytes;
use client::{Client, EventClient, EventStream};
use futures::prelude::*;
use http::{Method, Request, Response, Uri};
pub use lambda_attributes::lambda;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, env, error, fmt};

mod client;
/// Types availible to a Lambda function.
mod types;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// a string error
#[derive(Debug)]
pub(crate) struct StringError(pub String);

impl std::error::Error for StringError {}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! err_fmt {
    {$($t:tt)*} => {
        $crate::StringError(format!($($t)*))
    }
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

/// A trait describing an asynchronous function from `Event` to `Output`. `Event` and `Output` must implement [`Deserialize`](serde::Deserialize) and [`Serialize`](serde::Serialize).
pub trait Handler<Event, Output>
where
    Event: for<'de> Deserialize<'de>,
    Output: Serialize,
{
    /// Errors returned by this handler.
    type Err: Into<Error>;
    /// The future response value of this handler.
    type Fut: Future<Output = Result<Output, Self::Err>>;
    /// Process the incoming event and return the response asynchronously.
    ///
    /// # Arguments
    /// * `event` - The data received in the invocation request
    /// * `ctx` - The context for the current invocation
    fn call(&mut self, event: Event, ctx: Option<LambdaCtx>) -> Self::Fut;
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
    Err: Into<Error>,
    Fut: Future<Output = Result<Output, Err>> + Send,
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
/// #![feature(async_await)]
///
/// use lambda::{handler_fn, LambdaCtx, Error};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Err> {
///     let func = handler_fn(func);
///     lambda::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: String, _ctx: LambdaCtx) -> Result<String, Err> {
///     Ok(event)
/// }
/// ```
pub async fn run<Function, Event, Output>(mut handler: Function) -> Result<(), Error>
where
    Function: Handler<Event, Output>,
    Event: for<'de> Deserialize<'de>,
    Output: Serialize,
{
    let uri = env::var("AWS_LAMBDA_RUNTIME_API")?.into();
    let uri = Uri::from_shared(uri)?;
    let client = Client::new(uri);
    let mut stream = EventStream::new(&client);

    while let Some(event) = stream.next().await {
        let (parts, body) = event?.into_parts();
        let mut ctx: LambdaCtx = LambdaCtx::try_from(parts.headers)?;
        ctx.env_config = Config::from_env()?;
        let body = serde_json::from_slice(&body)?;

        match handler.call(body, Some(ctx.clone())).await {
            Ok(res) => {
                let res = serde_json::to_vec(&res)?;
                let uri = format!("/runtime/invocation/{}/response", &ctx.id).parse::<Uri>()?;
                let req = Request::builder()
                    .uri(uri)
                    .method(Method::POST)
                    .body(Bytes::from(res))?;

                client.call(req).await?;
            }
            Err(err) => {
                let err = type_name_of_val(err);
                let uri = format!("/runtime/invocation/{}/error", &ctx.id).parse::<Uri>()?;
                let req = Request::builder()
                    .uri(uri)
                    .method(Method::POST)
                    .body(Bytes::from(err))?;

                client.call(req).await?;
            }
        }
    }

    Ok(())
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

#[derive(Debug, Clone)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for MyError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[tokio::test]
async fn get_next() -> Result<(), Error> {
    fn test_fn() -> Result<String, MyError> {
        Err(MyError)
    }

    let res = test_fn();
    if let Err(e) = res {
        println!("{}", type_name_of_val(e));
    }

    Ok(())
}
