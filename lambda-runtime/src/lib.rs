#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The mechanism available for defining a Lambda function is as follows:
//!
//! Create a type that conforms to the [`tower::Service`] trait. This type can
//! then be passed to the the `lambda_runtime::run` function, which launches
//! and runs the Lambda runtime.
use bytes::Bytes;
use futures::FutureExt;
use http_body_util::BodyExt;
use hyper::{body::Incoming, http::Request};
use lambda_runtime_api_client::{body::Body, BoxError, Client};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Debug, Display},
    future::Future,
    panic,
    sync::Arc,
};
use tokio_stream::{Stream, StreamExt};
pub use tower::{self, service_fn, Service};
use tower::{util::ServiceFn, ServiceExt};
use tracing::{error, trace, Instrument};

mod deserializer;
mod requests;
/// Utilities for Lambda Streaming functions.
pub mod streaming;
/// Types available to a Lambda function.
mod types;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest};
pub use types::{Context, FunctionResponse, IntoFunctionResponse, LambdaEvent, MetadataPrelude, StreamResponse};

use types::invoke_request_id;

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

struct Runtime {
    client: Client,
    config: RefConfig,
}

impl Runtime {
    async fn run<F, A, R, B, S, D, E>(
        &self,
        incoming: impl Stream<Item = Result<http::Response<Incoming>, Error>> + Send,
        mut handler: F,
    ) -> Result<(), BoxError>
    where
        F: Service<LambdaEvent<A>>,
        F::Future: Future<Output = Result<R, F::Error>>,
        F::Error: fmt::Debug + fmt::Display,
        A: for<'de> Deserialize<'de>,
        R: IntoFunctionResponse<B, S>,
        B: Serialize,
        S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
        D: Into<Bytes> + Send,
        E: Into<Error> + Send + Debug,
    {
        let client = &self.client;
        tokio::pin!(incoming);
        while let Some(next_event_response) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = next_event_response?;
            let (parts, body) = event.into_parts();
            let request_id = invoke_request_id(&parts.headers)?;

            #[cfg(debug_assertions)]
            if parts.status == http::StatusCode::NO_CONTENT {
                // Ignore the event if the status code is 204.
                // This is a way to keep the runtime alive when
                // there are no events pending to be processed.
                continue;
            }

            let ctx: Context = Context::new(request_id, self.config.clone(), &parts.headers)?;
            let request_span = ctx.request_span();

            // Group the handling in one future and instrument it with the span
            async {
                let body = body.collect().await?.to_bytes();
                trace!("response body - {}", std::str::from_utf8(&body)?);

                #[cfg(debug_assertions)]
                if parts.status.is_server_error() {
                    error!("Lambda Runtime server returned an unexpected error");
                    return Err(parts.status.to_string().into());
                }

                let lambda_event = match deserializer::deserialize(&body, ctx) {
                    Ok(lambda_event) => lambda_event,
                    Err(err) => {
                        let req = build_event_error_request(request_id, err)?;
                        client.call(req).await.expect("Unable to send response to Runtime APIs");
                        return Ok(());
                    }
                };

                let req = match handler.ready().await {
                    Ok(handler) => {
                        // Catches panics outside of a `Future`
                        let task = panic::catch_unwind(panic::AssertUnwindSafe(|| handler.call(lambda_event)));

                        let task = match task {
                            // Catches panics inside of the `Future`
                            Ok(task) => panic::AssertUnwindSafe(task).catch_unwind().await,
                            Err(err) => Err(err),
                        };

                        match task {
                            Ok(response) => match response {
                                Ok(response) => {
                                    trace!("Ok response from handler (run loop)");
                                    EventCompletionRequest::new(request_id, response).into_req()
                                }
                                Err(err) => build_event_error_request(request_id, err),
                            },
                            Err(err) => {
                                error!("{:?}", err);
                                let error_type = type_name_of_val(&err);
                                let msg = if let Some(msg) = err.downcast_ref::<&str>() {
                                    format!("Lambda panicked: {msg}")
                                } else {
                                    "Lambda panicked".to_string()
                                };
                                EventErrorRequest::new(request_id, error_type, &msg).into_req()
                            }
                        }
                    }
                    Err(err) => build_event_error_request(request_id, err),
                }?;

                client.call(req).await.expect("Unable to send response to Runtime APIs");
                Ok::<(), Error>(())
            }
            .instrument(request_span)
            .await?;
        }
        Ok(())
    }
}

fn incoming(client: &Client) -> impl Stream<Item = Result<http::Response<Incoming>, Error>> + Send + '_ {
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
    F: Service<LambdaEvent<A>>,
    F::Future: Future<Output = Result<R, F::Error>>,
    F::Error: fmt::Debug + fmt::Display,
    A: for<'de> Deserialize<'de>,
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    trace!("Loading config from env");
    let config = Config::from_env();
    let client = Client::builder().build().expect("Unable to create a runtime client");
    let runtime = Runtime {
        client,
        config: Arc::new(config),
    };

    let client = &runtime.client;
    let incoming = incoming(client);
    runtime.run(incoming, handler).await
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

fn build_event_error_request<T>(request_id: &str, err: T) -> Result<Request<Body>, Error>
where
    T: Display + Debug,
{
    error!("{:?}", err); // logs the error in CloudWatch
    let error_type = type_name_of_val(&err);
    let msg = format!("{err}");

    EventErrorRequest::new(request_id, error_type, &msg).into_req()
}

#[cfg(test)]
mod endpoint_tests {
    use crate::{
        incoming,
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        types::Diagnostic,
        Config, Error, Runtime,
    };
    use futures::future::BoxFuture;
    use http::{HeaderValue, StatusCode};
    use http_body_util::BodyExt;
    use httpmock::prelude::*;

    use lambda_runtime_api_client::Client;
    use std::{env, sync::Arc};
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_next_event() -> Result<(), Error> {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = NextEventRequest.into_req()?;
        let rsp = client.call(req).await.expect("Unable to send request");

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        assert_eq!(
            rsp.headers()["lambda-runtime-aws-request-id"],
            &HeaderValue::from_static(request_id)
        );
        assert_eq!(
            rsp.headers()["lambda-runtime-deadline-ms"],
            &HeaderValue::from_static(deadline)
        );

        let body = rsp.into_body().collect().await?.to_bytes();
        assert_eq!("{}", std::str::from_utf8(&body)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_ok_response() -> Result<(), Error> {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/response")
                .body("\"{}\"");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = EventCompletionRequest::new("156cb537-e2d4-11e8-9b34-d36013741fb9", "{}");
        let req = req.into_req()?;

        let rsp = client.call(req).await?;

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn test_error_response() -> Result<(), Error> {
        let diagnostic = Diagnostic {
            error_type: "InvalidEventDataError",
            error_message: "Error parsing event data",
        };
        let body = serde_json::to_string(&diagnostic)?;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/error")
                .header("lambda-runtime-function-error-type", "unhandled")
                .body(body);
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = EventErrorRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            diagnostic,
        };
        let req = req.into_req()?;
        let rsp = client.call(req).await?;

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn successful_end_to_end_run() -> Result<(), Error> {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let next_request = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });
        let next_response = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/response", request_id))
                .body("{}");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        async fn func(event: crate::LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, Error> {
            let (event, _) = event.into_parts();
            Ok(event)
        }
        let f = crate::service_fn(func);

        // set env vars needed to init Config if they are not already set in the environment
        if env::var("AWS_LAMBDA_RUNTIME_API").is_err() {
            env::set_var("AWS_LAMBDA_RUNTIME_API", server.base_url());
        }
        if env::var("AWS_LAMBDA_FUNCTION_NAME").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test_fn");
        }
        if env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "128");
        }
        if env::var("AWS_LAMBDA_FUNCTION_VERSION").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "1");
        }
        if env::var("AWS_LAMBDA_LOG_STREAM_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_STREAM_NAME", "test_stream");
        }
        if env::var("AWS_LAMBDA_LOG_GROUP_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_GROUP_NAME", "test_log");
        }
        let config = Config::from_env();

        let runtime = Runtime {
            client,
            config: Arc::new(config),
        };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.run(incoming, f).await?;

        next_request.assert_async().await;
        next_response.assert_async().await;
        Ok(())
    }

    async fn run_panicking_handler<F>(func: F) -> Result<(), Error>
    where
        F: FnMut(crate::LambdaEvent<serde_json::Value>) -> BoxFuture<'static, Result<serde_json::Value, Error>>,
    {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let next_request = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });

        let next_response = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/error", request_id))
                .header("lambda-runtime-function-error-type", "unhandled");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let f = crate::service_fn(func);

        let config = Arc::new(Config {
            function_name: "test_fn".to_string(),
            memory: 128,
            version: "1".to_string(),
            log_stream: "test_stream".to_string(),
            log_group: "test_log".to_string(),
        });

        let runtime = Runtime { client, config };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.run(incoming, f).await?;

        next_request.assert_async().await;
        next_response.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn panic_in_async_run() -> Result<(), Error> {
        run_panicking_handler(|_| Box::pin(async { panic!("This is intentionally here") })).await
    }

    #[tokio::test]
    async fn panic_outside_async_run() -> Result<(), Error> {
        run_panicking_handler(|_| {
            panic!("This is intentionally here");
        })
        .await
    }
}
