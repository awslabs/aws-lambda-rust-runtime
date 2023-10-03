use crate::types::ToStreamErrorTrailer;
use crate::{types::Diagnostic, Error, FunctionResponse, IntoFunctionResponse};
use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{Method, Request, Response, Uri};
use hyper::Body;
use lambda_runtime_api_client::build_request;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;
use tokio_stream::{Stream, StreamExt};

pub(crate) trait IntoRequest {
    fn into_req(self) -> Result<Request<Body>, Error>;
}

pub(crate) trait IntoResponse {
    fn into_rsp(self) -> Result<Response<Body>, Error>;
}

// /runtime/invocation/next
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct NextEventRequest;

impl IntoRequest for NextEventRequest {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let req = build_request()
            .method(Method::GET)
            .uri(Uri::from_static("/2018-06-01/runtime/invocation/next"))
            .body(Body::empty())?;
        Ok(req)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NextEventResponse<'a> {
    // lambda-runtime-aws-request-id
    pub request_id: &'a str,
    // lambda-runtime-deadline-ms
    pub deadline: u64,
    // lambda-runtime-invoked-function-arn
    pub arn: &'a str,
    // lambda-runtime-trace-id
    pub trace_id: &'a str,
    // the actual body,
    pub body: Vec<u8>,
}

impl<'a> IntoResponse for NextEventResponse<'a> {
    fn into_rsp(self) -> Result<Response<Body>, Error> {
        let rsp = Response::builder()
            .header("lambda-runtime-aws-request-id", self.request_id)
            .header("lambda-runtime-deadline-ms", self.deadline)
            .header("lambda-runtime-invoked-function-arn", self.arn)
            .header("lambda-runtime-trace-id", self.trace_id)
            .body(Body::from(self.body))?;
        Ok(rsp)
    }
}
#[test]
fn test_next_event_request() {
    let req = NextEventRequest;
    let req = req.into_req().unwrap();
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri(), &Uri::from_static("/2018-06-01/runtime/invocation/next"));
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}

// /runtime/invocation/{AwsRequestId}/response
pub(crate) struct EventCompletionRequest<'a, R, B, S, D, E>
where
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    pub(crate) request_id: &'a str,
    pub(crate) body: R,
    pub(crate) _unused_b: PhantomData<B>,
    pub(crate) _unused_s: PhantomData<S>,
}

impl<'a, R, B, S, D, E> IntoRequest for EventCompletionRequest<'a, R, B, S, D, E>
where
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    fn into_req(self) -> Result<Request<Body>, Error> {
        match self.body.into_response() {
            FunctionResponse::BufferedResponse(body) => {
                let uri = format!("/2018-06-01/runtime/invocation/{}/response", self.request_id);
                let uri = Uri::from_str(&uri)?;

                let body = serde_json::to_vec(&body)?;
                let body = Body::from(body);

                let req = build_request().method(Method::POST).uri(uri).body(body)?;
                Ok(req)
            }
            FunctionResponse::StreamingResponse(mut response) => {
                let uri = format!("/2018-06-01/runtime/invocation/{}/response", self.request_id);
                let uri = Uri::from_str(&uri)?;

                let mut builder = build_request().method(Method::POST).uri(uri);
                let req_headers = builder.headers_mut().unwrap();

                req_headers.insert("Transfer-Encoding", "chunked".parse()?);
                req_headers.insert("Lambda-Runtime-Function-Response-Mode", "streaming".parse()?);
                // Report midstream errors using error trailers.
                // See the details in Lambda Developer Doc: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html#runtimes-custom-response-streaming
                req_headers.append("Trailer", "Lambda-Runtime-Function-Error-Type".parse()?);
                req_headers.append("Trailer", "Lambda-Runtime-Function-Error-Body".parse()?);
                req_headers.insert(
                    "Content-Type",
                    "application/vnd.awslambda.http-integration-response".parse()?,
                );

                // default Content-Type
                let preloud_headers = &mut response.metadata_prelude.headers;
                preloud_headers
                    .entry(CONTENT_TYPE)
                    .or_insert("application/octet-stream".parse()?);

                let metadata_prelude = serde_json::to_string(&response.metadata_prelude)?;

                tracing::trace!(?metadata_prelude);

                let (mut tx, rx) = Body::channel();

                tokio::spawn(async move {
                    tx.send_data(metadata_prelude.into()).await.unwrap();
                    tx.send_data("\u{0}".repeat(8).into()).await.unwrap();

                    while let Some(chunk) = response.stream.next().await {
                        let chunk = match chunk {
                            Ok(chunk) => chunk.into(),
                            Err(err) => err.into().to_tailer().into(),
                        };
                        tx.send_data(chunk).await.unwrap();
                    }
                });

                let req = builder.body(rx)?;
                Ok(req)
            }
        }
    }
}

#[test]
fn test_event_completion_request() {
    let req = EventCompletionRequest {
        request_id: "id",
        body: "hello, world!",
        _unused_b: PhantomData::<&str>,
        _unused_s: PhantomData::<Body>,
    };
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/2018-06-01/runtime/invocation/id/response");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}

// /runtime/invocation/{AwsRequestId}/error
pub(crate) struct EventErrorRequest<'a> {
    pub(crate) request_id: &'a str,
    pub(crate) diagnostic: Diagnostic<'a>,
}

impl<'a> EventErrorRequest<'a> {
    pub(crate) fn new(request_id: &'a str, error_type: &'a str, error_message: &'a str) -> EventErrorRequest<'a> {
        EventErrorRequest {
            request_id,
            diagnostic: Diagnostic {
                error_type,
                error_message,
            },
        }
    }
}

impl<'a> IntoRequest for EventErrorRequest<'a> {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let uri = format!("/2018-06-01/runtime/invocation/{}/error", self.request_id);
        let uri = Uri::from_str(&uri)?;
        let body = serde_json::to_vec(&self.diagnostic)?;
        let body = Body::from(body);

        let req = build_request()
            .method(Method::POST)
            .uri(uri)
            .header("lambda-runtime-function-error-type", "unhandled")
            .body(body)?;
        Ok(req)
    }
}

#[test]
fn test_event_error_request() {
    let req = EventErrorRequest {
        request_id: "id",
        diagnostic: Diagnostic {
            error_type: "InvalidEventDataError",
            error_message: "Error parsing event data",
        },
    };
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/2018-06-01/runtime/invocation/id/error");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}

// /runtime/init/error
pub(crate) struct InitErrorRequest<'a> {
    pub(crate) diagnostic: Diagnostic<'a>,
}

impl<'a> InitErrorRequest<'a> {
    pub(crate) fn new(error_type: &'a str, error_message: &'a str) -> InitErrorRequest<'a> {
        InitErrorRequest {
            diagnostic: Diagnostic {
                error_type,
                error_message,
            },
        }
    }
}

impl<'a> IntoRequest for InitErrorRequest<'a> {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let uri = "/2018-06-01/runtime/init/error".to_string();
        let uri = Uri::from_str(&uri)?;
        let body = serde_json::to_vec(&self.diagnostic)?;
        let body = Body::from(body);

        let req = build_request()
            .method(Method::POST)
            .uri(uri)
            .header("lambda-runtime-function-error-type", "unhandled")
            .body(body)?;
        Ok(req)
    }
}

#[test]
fn test_init_error_request() {
    let req = InitErrorRequest {
        diagnostic: Diagnostic {
            error_type: "runtime.InitError",
            error_message: "SnapShot Runtime Hook Error",
        },
    };
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/2018-06-01/runtime/init/error");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}

pub(crate) struct RestoreNextRequest;

impl IntoRequest for RestoreNextRequest {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let req = build_request()
            .method(Method::GET)
            .uri(Uri::from_static("/2018-06-01/runtime/restore/next"))
            .body(Body::empty())?;
        Ok(req)
    }
}
#[test]
fn test_restore_next_event_request() {
    let req = RestoreNextRequest;
    let req = req.into_req().unwrap();
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri(), &Uri::from_static("/2018-06-01/runtime/restore/next"));
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}
