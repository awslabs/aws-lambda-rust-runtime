#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The mechanism available for defining a Lambda function is as follows:
//!
//! Create a type that conforms to the [`tower::Service`] trait. This type can
//! then be passed to the the `lambda_runtime::run` function, which launches
//! and runs the Lambda runtime.
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Debug},
    future::Future,
    sync::Arc,
};
use tokio_stream::Stream;
use tower::util::ServiceFn;
pub use tower::{self, service_fn, Service};

/// Diagnostic utilities to convert Rust types into Lambda Error types.
pub mod diagnostic;
pub use diagnostic::Diagnostic;

mod deserializer;
/// Tower middleware to be applied to runtime invocations.
pub mod layers;
mod requests;
mod runtime;
/// Utilities for Lambda Streaming functions.
pub mod streaming;

/// Utilities to initialize and use `tracing` and `tracing-subscriber` in Lambda Functions.
#[cfg(feature = "tracing")]
pub use lambda_runtime_api_client::tracing;

/// Types available to a Lambda function.
mod types;

use requests::EventErrorRequest;
pub use runtime::{LambdaInvocation, Runtime};
pub use types::{Context, FunctionResponse, IntoFunctionResponse, LambdaEvent, MetadataPrelude, StreamResponse};

/// Error type that lambdas may result in
pub type Error = lambda_runtime_api_client::BoxError;

/// Configuration derived from environment variables.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
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

type RefConfig = Arc<Config>;

impl Config {
    /// Attempts to read configuration from environment variables.
    pub fn from_env() -> Self {
        Config {
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").expect("Missing AWS_LAMBDA_FUNCTION_NAME env var"),
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
                .expect("Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var")
                .parse::<i32>()
                .expect("AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var is not <i32>"),
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").expect("Missing AWS_LAMBDA_FUNCTION_VERSION env var"),
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME").unwrap_or_default(),
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME").unwrap_or_default(),
        }
    }
}

/// Return a new [`ServiceFn`] with a closure that takes an event and context as separate arguments.
#[deprecated(since = "0.5.0", note = "Use `service_fn` and `LambdaEvent` instead")]
pub fn handler_fn<A, F, Fut>(f: F) -> ServiceFn<impl Fn(LambdaEvent<A>) -> Fut>
where
    F: Fn(A, Context) -> Fut,
{
    service_fn(move |req: LambdaEvent<A>| f(req.payload, req.context))
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// If you need more control over the runtime and add custom middleware, use the
/// [Runtime] type directly.
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, service_fn, LambdaEvent};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = service_fn(func);
///     lambda_runtime::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
///     Ok(event.payload)
/// }
/// ```
pub async fn run<A, F, R, B, S, D, E>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaEvent<A>, Response = R>,
    F::Future: Future<Output = Result<R, F::Error>>,
    F::Error: for<'a> Into<Diagnostic<'a>> + fmt::Debug,
    A: for<'de> Deserialize<'de>,
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<bytes::Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    let runtime = Runtime::new(handler).layer(layers::TracingLayer::new());
    runtime.run().await
}
