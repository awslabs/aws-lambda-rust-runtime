#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! The mechanism available for defining a Lambda function is as follows:
//!
//! Create a type that conforms to the [`tower::Service`] trait. This type can
//! then be passed to the the `lambda_runtime::run` function, which launches
//! and runs the Lambda runtime.
use futures::FutureExt;
use hyper::{
    client::{connect::Connection, HttpConnector},
    http::Request,
    Body,
};
use lambda_runtime_api_client::Client;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    env,
    fmt::{self, Debug, Display},
    future::Future,
    panic,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::{Stream, StreamExt};
pub use tower::{self, service_fn, Service};
use tower::{util::ServiceFn, ServiceExt};
use tracing::{error, trace, Instrument};

mod deserializer;
mod requests;
#[cfg(test)]
mod simulated;
/// Types available to a Lambda function.
mod types;

mod streaming;
pub use streaming::run_with_streaming_response;

use requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest};
pub use types::{Context, LambdaEvent};

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

struct Runtime<C: Service<http::Uri> = HttpConnector> {
    client: Client<C>,
    config: Config,
}

impl<C> Runtime<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn run<F, A, B>(
        &self,
        incoming: impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send,
        mut handler: F,
    ) -> Result<(), Error>
    where
        F: Service<LambdaEvent<A>>,
        F::Future: Future<Output = Result<B, F::Error>>,
        F::Error: fmt::Debug + fmt::Display,
        A: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let client = &self.client;
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
}

fn incoming<C>(client: &Client<C>) -> impl Stream<Item = Result<http::Response<hyper::Body>, Error>> + Send + '_
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    <C as Service<http::Uri>>::Future: Unpin + Send,
    <C as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    <C as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
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
pub async fn run<A, B, F>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaEvent<A>>,
    F::Future: Future<Output = Result<B, F::Error>>,
    F::Error: fmt::Debug + fmt::Display,
    A: for<'de> Deserialize<'de>,
    B: Serialize,
{
    trace!("Loading config from env");
    let config = Config::from_env()?;
    let client = Client::builder().build().expect("Unable to create a runtime client");
    let runtime = Runtime { client, config };

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
        requests::{
            EventCompletionRequest, EventErrorRequest, IntoRequest, IntoResponse, NextEventRequest, NextEventResponse,
        },
        simulated,
        types::Diagnostic,
        Error, Runtime,
    };
    use futures::future::BoxFuture;
    use http::{uri::PathAndQuery, HeaderValue, Method, Request, Response, StatusCode, Uri};
    use hyper::{server::conn::Http, service::service_fn, Body};
    use lambda_runtime_api_client::Client;
    use serde_json::json;
    use simulated::DuplexStreamWrapper;
    use std::{convert::TryFrom, env};
    use tokio::{
        io::{self, AsyncRead, AsyncWrite},
        select,
        sync::{self, oneshot},
    };
    use tokio_stream::StreamExt;

    #[cfg(test)]
    async fn next_event(req: &Request<Body>) -> Result<Response<Body>, Error> {
        let path = "/2018-06-01/runtime/invocation/next";
        assert_eq!(req.method(), Method::GET);
        assert_eq!(req.uri().path_and_query().unwrap(), &PathAndQuery::from_static(path));
        let body = json!({"message": "hello"});

        let rsp = NextEventResponse {
            request_id: "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
            deadline: 1_542_409_706_888,
            arn: "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
            trace_id: "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419",
            body: serde_json::to_vec(&body)?,
        };
        rsp.into_rsp()
    }

    #[cfg(test)]
    async fn complete_event(req: &Request<Body>, id: &str) -> Result<Response<Body>, Error> {
        assert_eq!(Method::POST, req.method());
        let rsp = Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(Body::empty())
            .expect("Unable to construct response");

        let expected = format!("/2018-06-01/runtime/invocation/{id}/response");
        assert_eq!(expected, req.uri().path());

        Ok(rsp)
    }

    #[cfg(test)]
    async fn event_err(req: &Request<Body>, id: &str) -> Result<Response<Body>, Error> {
        let expected = format!("/2018-06-01/runtime/invocation/{id}/error");
        assert_eq!(expected, req.uri().path());

        assert_eq!(req.method(), Method::POST);
        let header = "lambda-runtime-function-error-type";
        let expected = "unhandled";
        assert_eq!(req.headers()[header], HeaderValue::try_from(expected)?);

        let rsp = Response::builder().status(StatusCode::ACCEPTED).body(Body::empty())?;
        Ok(rsp)
    }

    #[cfg(test)]
    async fn handle_incoming(req: Request<Body>) -> Result<Response<Body>, Error> {
        let path: Vec<&str> = req
            .uri()
            .path_and_query()
            .expect("PathAndQuery not found")
            .as_str()
            .split('/')
            .collect::<Vec<&str>>();
        match path[1..] {
            ["2018-06-01", "runtime", "invocation", "next"] => next_event(&req).await,
            ["2018-06-01", "runtime", "invocation", id, "response"] => complete_event(&req, id).await,
            ["2018-06-01", "runtime", "invocation", id, "error"] => event_err(&req, id).await,
            ["2018-06-01", "runtime", "init", "error"] => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    #[cfg(test)]
    async fn handle<I>(io: I, rx: oneshot::Receiver<()>) -> Result<(), hyper::Error>
    where
        I: AsyncRead + AsyncWrite + Unpin + 'static,
    {
        let conn = Http::new().serve_connection(io, service_fn(handle_incoming));
        select! {
            _ = rx => {
                Ok(())
            }
            res = conn => {
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        Err(e)
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_next_event() -> Result<(), Error> {
        let base = Uri::from_static("http://localhost:9001");
        let (client, server) = io::duplex(64);

        let (tx, rx) = sync::oneshot::channel();
        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

        let req = NextEventRequest.into_req()?;
        let rsp = client.call(req).await.expect("Unable to send request");

        assert_eq!(rsp.status(), StatusCode::OK);
        let header = "lambda-runtime-deadline-ms";
        assert_eq!(rsp.headers()[header], &HeaderValue::try_from("1542409706888")?);

        // shutdown server...
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn test_ok_response() -> Result<(), Error> {
        let (client, server) = io::duplex(64);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

        let req = EventCompletionRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            body: "done",
        };
        let req = req.into_req()?;

        let rsp = client.call(req).await?;
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn test_error_response() -> Result<(), Error> {
        let (client, server) = io::duplex(200);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

        let req = EventErrorRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            diagnostic: Diagnostic {
                error_type: "InvalidEventDataError",
                error_message: "Error parsing event data",
            },
        };
        let req = req.into_req()?;
        let rsp = client.call(req).await?;
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn successful_end_to_end_run() -> Result<(), Error> {
        let (client, server) = io::duplex(64);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });
        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;

        let client = Client::builder()
            .with_endpoint(base)
            .with_connector(conn)
            .build()
            .expect("Unable to build client");

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

        let runtime = Runtime { client, config };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.run(incoming, f).await?;

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    async fn run_panicking_handler<F>(func: F) -> Result<(), Error>
    where
        F: FnMut(crate::LambdaEvent<serde_json::Value>) -> BoxFuture<'static, Result<serde_json::Value, Error>>,
    {
        let (client, server) = io::duplex(64);
        let (_tx, rx) = oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });
        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;

        let client = Client::builder()
            .with_endpoint(base)
            .with_connector(conn)
            .build()
            .expect("Unable to build client");

        let f = crate::service_fn(func);

        let config = crate::Config {
            function_name: "test_fn".to_string(),
            memory: 128,
            version: "1".to_string(),
            log_stream: "test_stream".to_string(),
            log_group: "test_log".to_string(),
        };

        let runtime = Runtime { client, config };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.run(incoming, f).await?;

        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
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
