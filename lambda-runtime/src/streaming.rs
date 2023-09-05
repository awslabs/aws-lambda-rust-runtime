use crate::{
    build_event_error_request, deserializer, incoming, type_name_of_val, Config, Context, Error, EventErrorRequest,
    IntoRequest, LambdaEvent, Runtime,
};
use bytes::Bytes;
use futures::FutureExt;
use http::header::{CONTENT_TYPE, SET_COOKIE};
use http::{HeaderMap, Method, Request, Response, StatusCode, Uri};
use hyper::body::HttpBody;
use hyper::{client::connect::Connection, Body};
use lambda_runtime_api_client::{build_request, Client};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{
    env,
    fmt::{self, Debug, Display},
    future::Future,
    panic,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::{Stream, StreamExt};
use tower::{Service, ServiceExt};
use tracing::{error, trace, Instrument};

/// Starts the Lambda Rust runtime and stream response back [Configure Lambda
/// Streaming Response](https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html).
///
/// # Example
/// ```no_run
/// use hyper::{body::Body, Response};
/// use lambda_runtime::{service_fn, Error, LambdaEvent};
/// use std::{thread, time::Duration};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     lambda_runtime::run_with_streaming_response(service_fn(func)).await?;
///     Ok(())
/// }
/// async fn func(_event: LambdaEvent<Value>) -> Result<Response<Body>, Error> {
///     let messages = vec!["Hello ", "world ", "from ", "Lambda!"];
///
///     let (mut tx, rx) = Body::channel();
///
///     tokio::spawn(async move {
///         for message in messages.iter() {
///             tx.send_data((*message).into()).await.unwrap();
///             thread::sleep(Duration::from_millis(500));
///         }
///     });
///
///     let resp = Response::builder()
///         .header("content-type", "text/plain")
///         .header("CustomHeader", "outerspace")
///         .body(rx)?;
///     
///     Ok(resp)
/// }
/// ```
pub async fn run_with_streaming_response<A, B, F>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaEvent<A>>,
    F::Future: Future<Output = Result<http::Response<B>, F::Error>>,
    F::Error: Debug + Display,
    A: for<'de> Deserialize<'de>,
    B: HttpBody + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    trace!("Loading config from env");
    let config = Config::from_env()?;
    let client = Client::builder().build().expect("Unable to create a runtime client");
    let runtime = Runtime { client, config };

    let client = &runtime.client;
    let incoming = incoming(client);
    runtime.run_with_streaming_response(incoming, handler).await
}

impl<C> Runtime<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn run_with_streaming_response<F, A, B>(
        &self,
        incoming: impl Stream<Item = Result<Response<Body>, Error>> + Send,
        mut handler: F,
    ) -> Result<(), Error>
    where
        F: Service<LambdaEvent<A>>,
        F::Future: Future<Output = Result<Response<B>, F::Error>>,
        F::Error: fmt::Debug + fmt::Display,
        A: for<'de> Deserialize<'de>,
        B: HttpBody + Unpin + Send + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Error> + Send + Debug,
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
                trace!("incoming request payload - {}", std::str::from_utf8(&body)?);

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
                                    EventCompletionStreamingRequest {
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

pub(crate) struct EventCompletionStreamingRequest<'a, B> {
    pub(crate) request_id: &'a str,
    pub(crate) body: Response<B>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataPrelude {
    #[serde(serialize_with = "http_serde::status_code::serialize")]
    status_code: StatusCode,
    #[serde(serialize_with = "http_serde::header_map::serialize")]
    headers: HeaderMap,
    cookies: Vec<String>,
}

impl<'a, B> IntoRequest for EventCompletionStreamingRequest<'a, B>
where
    B: HttpBody + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    fn into_req(self) -> Result<Request<Body>, Error> {
        let uri = format!("/2018-06-01/runtime/invocation/{}/response", self.request_id);
        let uri = Uri::from_str(&uri)?;

        let (parts, mut body) = self.body.into_parts();

        let mut builder = build_request().method(Method::POST).uri(uri);
        let req_headers = builder.headers_mut().unwrap();

        req_headers.insert("Transfer-Encoding", "chunked".parse()?);
        req_headers.insert("Lambda-Runtime-Function-Response-Mode", "streaming".parse()?);
        req_headers.insert(
            "Content-Type",
            "application/vnd.awslambda.http-integration-response".parse()?,
        );

        let mut prelude_headers = parts.headers;
        // default Content-Type
        prelude_headers
            .entry(CONTENT_TYPE)
            .or_insert("application/octet-stream".parse()?);

        let cookies = prelude_headers.get_all(SET_COOKIE);
        let cookies = cookies
            .iter()
            .map(|c| String::from_utf8_lossy(c.as_bytes()).to_string())
            .collect::<Vec<String>>();
        prelude_headers.remove(SET_COOKIE);

        let metadata_prelude = serde_json::to_string(&MetadataPrelude {
            status_code: parts.status,
            headers: prelude_headers,
            cookies,
        })?;

        trace!(?metadata_prelude);

        let (mut tx, rx) = Body::channel();

        tokio::spawn(async move {
            tx.send_data(metadata_prelude.into()).await.unwrap();
            tx.send_data("\u{0}".repeat(8).into()).await.unwrap();

            while let Some(chunk) = body.data().await {
                let chunk = chunk.unwrap();
                tx.send_data(chunk.into()).await.unwrap();
            }
        });

        let req = builder.body(rx)?;
        Ok(req)
    }
}
