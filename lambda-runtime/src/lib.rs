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
use hyper::{client::HttpConnector, http::Request, Body};
use lambda_runtime_api_client::Client;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    env,
    fmt::{self, Debug, Display},
    future::Future,
    marker::PhantomData,
    panic,
};
use tokio_stream::{Stream, StreamExt};
pub use tower::{self, service_fn, Service};
use tower::{util::ServiceFn, ServiceExt};
use tracing::{error, trace, Instrument};

/// CRAC module contains the Resource trait for receiving checkpoint/restore notifications.
pub mod crac;
mod deserializer;
mod requests;
/// Types available to a Lambda function.
mod types;

use requests::{
    EventCompletionRequest, EventErrorRequest, InitErrorRequest, IntoRequest, NextEventRequest, RestoreNextRequest,
};
pub use types::{Context, FunctionResponse, IntoFunctionResponse, LambdaEvent, MetadataPrelude, StreamResponse};

/// Error type that lambdas may result in
pub type Error = lambda_runtime_api_client::Error;

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
    /// The initialization type of the functin, which is 'on-demand', 'provisioned-concurrency', or 'snap-start'.
    pub init_type: String,
}

impl Config {
    /// Attempts to read configuration from environment variables.
    pub fn from_env() -> Result<Self, Error> {
        let conf = Config {
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").expect("Missing AWS_LAMBDA_FUNCTION_NAME env var"),
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
                .expect("Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var")
                .parse::<i32>()
                .expect("AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var is not <i32>"),
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").expect("Missing AWS_LAMBDA_FUNCTION_VERSION env var"),
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME").unwrap_or_default(),
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME").unwrap_or_default(),
            init_type: env::var("AWS_LAMBDA_INITIALIZATION_TYPE").unwrap_or_default(),
        };
        Ok(conf)
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

/// The entry point for the lambda function using Builder pattern.
pub struct Runtime<'a, T: crac::Resource> {
    client: Client<HttpConnector>,
    config: Config,
    crac_context: crac::Context<'a, T>,
}

impl<'a, T: crac::Resource> Runtime<'a, T> {
    async fn execute<F, A, R, B, S, D, E>(
        &self,
        incoming: impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send,
        mut handler: F,
    ) -> Result<(), Error>
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
        if "snap-start" == self.config.init_type {
            self.on_init_complete(client).await?;
        }
        tokio::pin!(incoming);
        while let Some(next_event_response) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = next_event_response?;
            let (parts, body) = event.into_parts();

            #[cfg(debug_assertions)]
            if parts.status == http::StatusCode::NO_CONTENT {
                // Ignore the event if the status code is 204.
                // This is a way to keep the runtime alive when
                // there are no events pending to be processed.
                continue;
            }

            let ctx: Context = Context::try_from(parts.headers)?;
            let ctx: Context = ctx.with_config(&self.config);
            let request_id = &ctx.request_id.clone();

            let request_span = match &ctx.xray_trace_id {
                Some(trace_id) => {
                    env::set_var("_X_AMZN_TRACE_ID", trace_id);
                    tracing::info_span!("Lambda runtime invoke", requestId = request_id, xrayTraceId = trace_id)
                }
                None => {
                    env::remove_var("_X_AMZN_TRACE_ID");
                    tracing::info_span!("Lambda runtime invoke", requestId = request_id)
                }
            };

            // Group the handling in one future and instrument it with the span
            async {
                let body = hyper::body::to_bytes(body).await?;
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
                                    EventCompletionRequest {
                                        request_id,
                                        body: response,
                                        _unused_b: PhantomData,
                                        _unused_s: PhantomData,
                                    }
                                    .into_req()
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

    async fn on_init_complete(&self, client: &Client<HttpConnector>) -> Result<(), Error> {
        let res = self.crac_context.before_checkpoint();
        if let Err(err) = res {
            error!("{:?}", err);
            let req = InitErrorRequest::new("runtime.BeforeCheckpointError", &err.to_string()).into_req()?;
            client.call(req).await?;
            std::process::exit(64)
        }

        let req = RestoreNextRequest
            .into_req()
            .expect("Unable to build restore next requests");
        // Blocking call to RAPID /runtime/restore/next API, will return after taking snapshot.
        // This will also be the 'entrypoint' when resuming from snapshots.
        client.call(req).await?;

        let res = self.crac_context.after_restore();
        if let Err(err) = res {
            error!("{:?}", err);
            let req = InitErrorRequest::new("runtime.AfterRestoreError", &err.to_string()).into_req()?;
            client.call(req).await?;
            std::process::exit(64)
        }

        Ok(())
    }

    /// Creates a new Lambda Rust runtime.
    pub fn new() -> Self {
        trace!("Loading config from env");
        let config = Config::from_env().expect("Unable to parse config from environment variables");
        let client = Client::builder().build().expect("Unable to create a runtime client");
        Runtime {
            client,
            config,
            crac_context: crac::Context::new(),
        }
    }

    /// Registers a crac::Resource with the runtime.
    pub fn register(&mut self, resource: &'a T) -> &mut Self {
        self.crac_context.register(resource);
        self
    }

    /// Runs the Lambda Rust runtime.
    pub async fn run<A, F, R, B, S, D, E>(&self, handler: F) -> Result<(), Error>
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
        let incoming = incoming(&self.client);
        self.execute(incoming, handler).await
    }
}

fn incoming(
    client: &Client<HttpConnector>,
) -> impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send + '_ {
    async_stream::stream! {
        loop {
            trace!("Waiting for next event (incoming loop)");
            let req = NextEventRequest.into_req().expect("Unable to construct request");
            let res = client.call(req).await;
            yield res;
        }
    }
}

impl<'a, T: crac::Resource> Default for Runtime<'a, T> {
    fn default() -> Self {
        Runtime::new()
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
    Runtime::new().register(&()).run(handler).await
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
        crac, incoming,
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        types::Diagnostic,
        Error, Runtime,
    };
    use futures::future::BoxFuture;
    use http::{HeaderValue, StatusCode, Uri};
    use httpmock::prelude::*;
    use hyper::{client::HttpConnector, Body};
    use lambda_runtime_api_client::Client;
    use serde_json::json;
    use std::{convert::TryFrom, env, marker::PhantomData};
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_next_event() {
        // Start mock runtime api server
        let rapid = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let endpoint = rapid.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(StatusCode::OK.as_u16())
                .header("Lambda-Runtime-Aws-Request-Id", request_id)
                .header("Lambda-Runtime-Deadline-Ms", "1542409706888")
                .body("ok");
        });

        // build the next event request and send to the mock endpoint
        let base = Uri::try_from(format!("http://{}", rapid.address())).unwrap();
        let client = Client::with(base, HttpConnector::new());
        let req = NextEventRequest.into_req().unwrap();
        let rsp = client.call(req).await.expect("Unable to send request");

        // Assert endpoint was called once
        endpoint.assert();
        // and response has expected content
        assert_eq!(rsp.status(), StatusCode::OK);
        assert_eq!(
            rsp.headers()["Lambda-Runtime-Aws-Request-Id"],
            &HeaderValue::try_from(request_id).unwrap()
        );
        assert_eq!(
            rsp.headers()["Lambda-Runtime-Deadline-Ms"],
            &HeaderValue::try_from("1542409706888").unwrap()
        );
    }

    #[tokio::test]
    async fn test_ok_response() {
        // Start mock runtime api server
        let rapid = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let endpoint = rapid.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/response", request_id));
            then.status(StatusCode::ACCEPTED.as_u16());
        });

        // build the OK response and send to the mock endpoint
        let base = Uri::try_from(format!("http://{}", rapid.address())).unwrap();
        let client = Client::with(base, HttpConnector::new());

        let req = EventCompletionRequest {
            request_id,
            body: "done",
            _unused_b: PhantomData::<&str>,
            _unused_s: PhantomData::<Body>,
        };
        let req = req.into_req().unwrap();

        let rsp = client.call(req).await.unwrap();

        // Assert endpoint was called once
        endpoint.assert();
        // and response has expected content
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_error_response() {
        // Start mock runtime api server
        let rapid = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let endpoint = rapid.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/error", request_id));
            then.status(StatusCode::ACCEPTED.as_u16());
        });

        // build the ERROR response and send to the mock endpoint
        let base = Uri::try_from(format!("http://{}", rapid.address())).unwrap();
        let client = Client::with(base, HttpConnector::new());

        let req = EventErrorRequest {
            request_id,
            diagnostic: Diagnostic {
                error_type: "InvalidEventDataError",
                error_message: "Error parsing event data",
            },
        };
        let req = req.into_req().unwrap();
        let rsp = client.call(req).await.unwrap();

        // Assert endpoint was called once
        endpoint.assert();
        // and response has expected content
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn successful_end_to_end_run() {
        // Start mock runtime api server
        let rapid = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let next_endpoint = rapid.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(StatusCode::OK.as_u16())
                .header("Lambda-Runtime-Aws-Request-Id", request_id)
                .header("Lambda-Runtime-Deadline-Ms", "1542409706888")
                .body(json!({"command": "hello"}).to_string());
        });
        let response_endpoint = rapid.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/response", request_id));
            then.status(StatusCode::ACCEPTED.as_u16());
        });

        // build the client to the mock endpoint
        let base = Uri::try_from(format!("http://{}", rapid.address())).unwrap();
        let client = Client::with(base, HttpConnector::new());

        async fn func(event: crate::LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, Error> {
            let (event, _) = event.into_parts();
            Ok(event)
        }
        let f = crate::service_fn(func);

        // set env vars needed to init Config if they are not already set in the environment
        if env::var("AWS_LAMBDA_RUNTIME_API").is_err() {
            env::set_var("AWS_LAMBDA_RUNTIME_API", "http://localhost:9001");
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
        let config = crate::Config::from_env().expect("Failed to read env vars");

        let runtime = Runtime {
            client,
            config,
            crac_context: crac::Context::<()>::new(),
        };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.execute(incoming, f).await.unwrap();

        // Assert endpoints were called
        next_endpoint.assert();
        response_endpoint.assert();
    }

    async fn run_panicking_handler<F>(func: F)
    where
        F: FnMut(crate::LambdaEvent<serde_json::Value>) -> BoxFuture<'static, Result<serde_json::Value, Error>>,
    {
        // Start mock runtime api server
        let rapid = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let next_endpoint = rapid.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(StatusCode::OK.as_u16())
                .header("Lambda-Runtime-Aws-Request-Id", request_id)
                .header("Lambda-Runtime-Deadline-Ms", "1542409706888")
                .body(json!({"command": "hello"}).to_string());
        });
        let error_endpoint = rapid.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{}/error", request_id));
            then.status(StatusCode::ACCEPTED.as_u16());
        });

        // build the client to the mock endpoint
        let base = Uri::try_from(format!("http://{}", rapid.address())).unwrap();
        let client = Client::with(base, HttpConnector::new());

        let f = crate::service_fn(func);

        let config = crate::Config {
            function_name: "test_fn".to_string(),
            memory: 128,
            version: "1".to_string(),
            log_stream: "test_stream".to_string(),
            log_group: "test_log".to_string(),
            init_type: "on-demand".to_string(),
        };

        let runtime = Runtime {
            client,
            config,
            crac_context: crac::Context::<()>::new(),
        };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.execute(incoming, f).await.unwrap();

        // Assert endpoints were called
        next_endpoint.assert();
        error_endpoint.assert();
    }

    #[tokio::test]
    async fn panic_in_async_run() {
        run_panicking_handler(|_| Box::pin(async { panic!("This is intentionally here") })).await
    }

    #[tokio::test]
    async fn panic_outside_async_run() {
        run_panicking_handler(|_| {
            panic!("This is intentionally here");
        })
        .await
    }

    #[tokio::test]
    async fn test_snapstart_runtime_hooks() {}
}
