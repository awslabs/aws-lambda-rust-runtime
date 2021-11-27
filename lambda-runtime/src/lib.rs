#![deny(clippy::all, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The mechanism available for defining a Lambda function is as follows:
//!
//! Create a type that conforms to the [`Handler`] trait. This type can then be passed
//! to the the `lambda_runtime::run` function, which launches and runs the Lambda runtime.
pub use crate::types::Context;
use client::Client;
use hyper::client::{connect::Connection, HttpConnector};
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    env, fmt,
    future::Future,
    panic,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::{Stream, StreamExt};
pub use tower;
use tracing::{error, trace};

use tower::{service_fn, util::ServiceFn, Service};

mod client;
mod requests;
#[cfg(test)]
mod simulated;
/// Types available to a Lambda function.
mod types;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest};
use types::Diagnostic;

pub use types::LambdaRequest;

/// Error type that lambdas may result in
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Configuration derived from environment variables.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
            endpoint: env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var"),
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").expect("Missing AWS_LAMBDA_FUNCTION_NAME env var"),
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
                .expect("Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var")
                .parse::<i32>()
                .expect("AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var is not <i32>"),
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").expect("Missing AWS_LAMBDA_FUNCTION_VERSION env var"),
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME").expect("Missing AWS_LAMBDA_LOG_STREAM_NAME env var"),
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME").expect("Missing AWS_LAMBDA_LOG_GROUP_NAME env var"),
        };
        Ok(conf)
    }
}

/// Wraps a function that takes 2 arguments into one that only takes a [`LambdaRequest`].
fn handler_wrapper<A, Fut>(f: impl Fn(A, Context) -> Fut) -> impl Fn(LambdaRequest<A>) -> Fut {
    move |req| f(req.event, req.context)
}

/// Return a new [`ServiceFn`] with a closure that takes an event and context as separate arguments.
pub fn handler_fn<A, F, Fut>(f: F) -> ServiceFn<impl Fn(LambdaRequest<A>) -> Fut>
where
    F: Fn(A, Context) -> Fut,
{
    service_fn(handler_wrapper(f))
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
enum BuilderError {
    UnsetUri,
}

struct Runtime<C: Service<http::Uri> = HttpConnector> {
    client: Client<C>,
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder<HttpConnector> {
        RuntimeBuilder {
            connector: HttpConnector::new(),
            uri: None,
        }
    }
}

impl<C> Runtime<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn run<F, A, B>(
        &self,
        incoming: impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send,
        mut handler: F,
        config: &Config,
    ) -> Result<(), Error>
    where
        F: Service<LambdaRequest<A>>,
        F::Future: Future<Output = Result<B, F::Error>>,
        F::Error: fmt::Display,
        A: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let client = &self.client;
        tokio::pin!(incoming);
        while let Some(event) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = event?;
            let (parts, body) = event.into_parts();

            let ctx: Context = Context::try_from(parts.headers)?;
            let ctx: Context = ctx.with_config(config);
            let body = hyper::body::to_bytes(body).await?;
            trace!("{}", std::str::from_utf8(&body)?); // this may be very verbose
            let body = serde_json::from_slice(&body)?;

            let xray_trace_id = &ctx.xray_trace_id.clone();
            env::set_var("_X_AMZN_TRACE_ID", xray_trace_id);

            let request_id = &ctx.request_id.clone();
            let task = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                handler.call(LambdaRequest {
                    event: body,
                    context: ctx,
                })
            }));

            let req = match task {
                Ok(response) => match response.await {
                    Ok(response) => {
                        trace!("Ok response from handler (run loop)");
                        EventCompletionRequest {
                            request_id,
                            body: response,
                        }
                        .into_req()
                    }
                    Err(err) => {
                        error!("{}", err); // logs the error in CloudWatch
                        EventErrorRequest {
                            request_id,
                            diagnostic: Diagnostic {
                                error_type: type_name_of_val(&err).to_owned(),
                                error_message: format!("{}", err), // returns the error to the caller via Lambda API
                            },
                        }
                        .into_req()
                    }
                },
                Err(err) => {
                    error!("{:?}", err); // inconsistent with other log record formats - to be reviewed
                    EventErrorRequest {
                        request_id,
                        diagnostic: Diagnostic {
                            error_type: type_name_of_val(&err).to_owned(),
                            error_message: if let Some(msg) = err.downcast_ref::<&str>() {
                                format!("Lambda panicked: {}", msg)
                            } else {
                                "Lambda panicked".to_string()
                            },
                        },
                    }
                    .into_req()
                }
            };
            let req = req?;
            client.call(req).await.expect("Unable to send response to Runtime APIs");
        }
        Ok(())
    }
}

struct RuntimeBuilder<C: Service<http::Uri> = hyper::client::HttpConnector> {
    connector: C,
    uri: Option<http::Uri>,
}

impl<C> RuntimeBuilder<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub fn with_connector<C2>(self, connector: C2) -> RuntimeBuilder<C2>
    where
        C2: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
        C2::Future: Unpin + Send,
        C2::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        C2::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    {
        RuntimeBuilder {
            connector,
            uri: self.uri,
        }
    }

    pub fn with_endpoint(self, uri: http::Uri) -> Self {
        Self { uri: Some(uri), ..self }
    }

    pub fn build(self) -> Result<Runtime<C>, BuilderError> {
        let uri = match self.uri {
            Some(uri) => uri,
            None => return Err(BuilderError::UnsetUri),
        };
        let client = Client::with(uri, self.connector);

        Ok(Runtime { client })
    }
}

#[test]
fn test_builder() {
    let runtime = Runtime::builder()
        .with_connector(HttpConnector::new())
        .with_endpoint(http::Uri::from_static("http://nomatter.com"))
        .build();

    runtime.unwrap();
}

fn incoming<C>(client: &Client<C>) -> impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send + '_
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async_stream::stream! {
        loop {
            trace!("Waiting for next event (incoming loop)");
            let req = NextEventRequest.into_req().expect("Unable to construct request");
            let res = client.call(req).await;
            yield res;
        }
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, handler_fn, Context};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = handler_fn(func);
///     lambda_runtime::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: Value, _: Context) -> Result<Value, Error> {
///     Ok(event)
/// }
/// ```
pub async fn run<A, B, F>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaRequest<A>>,
    F::Future: Future<Output = Result<B, F::Error>>,
    F::Error: fmt::Display,
    A: for<'de> Deserialize<'de>,
    B: Serialize,
{
    trace!("Loading config from env");
    let config = Config::from_env()?;
    let uri = config.endpoint.clone().try_into().expect("Unable to convert to URL");
    let runtime = Runtime::builder()
        .with_connector(HttpConnector::new())
        .with_endpoint(uri)
        .build()
        .expect("Unable to create a runtime");

    let client = &runtime.client;
    let incoming = incoming(client);
    runtime.run(incoming, handler, &config).await
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}
