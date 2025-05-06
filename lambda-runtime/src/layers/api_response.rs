use crate::{
    deserializer,
    requests::{EventCompletionRequest, IntoRequest},
    runtime::LambdaInvocation,
    Diagnostic, EventErrorRequest, IntoFunctionResponse, LambdaEvent,
};
use futures::{ready, Stream};
use lambda_runtime_api_client::{body::Body, BoxError};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task};
use tower::Service;
use tracing::{error, trace};

/// Tower service that turns the result or an error of a handler function into a Lambda Runtime API
/// response.
///
/// This type is only meant for internal use in the Lambda runtime crate. The service augments both
/// inputs and outputs: the input is converted from a [LambdaInvocation] into a [LambdaEvent]
/// while any errors encountered during the conversion are turned into error responses. The service
/// outputs either a HTTP request to send to the Lambda Runtime API or a boxed error which ought to
/// be propagated to the caller to terminate the runtime.
pub struct RuntimeApiResponseService<
    S,
    EventPayload,
    Response,
    BufferedResponse,
    StreamingResponse,
    StreamItem,
    StreamError,
> {
    inner: S,
    _phantom: PhantomData<(
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    )>,
}

impl<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
    RuntimeApiResponseService<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
{
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> Service<LambdaInvocation>
    for RuntimeApiResponseService<
        S,
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    >
where
    S: Service<LambdaEvent<EventPayload>, Response = Response, Error = Diagnostic>,
    EventPayload: for<'de> Deserialize<'de>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    type Response = http::Request<Body>;
    type Error = BoxError;
    type Future =
        RuntimeApiResponseFuture<S::Future, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|err| BoxError::from(format!("{}: {}", err.error_type, err.error_message)))
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        #[cfg(debug_assertions)]
        if req.parts.status.is_server_error() {
            error!("Lambda Runtime server returned an unexpected error");
            return RuntimeApiResponseFuture::Ready(Some(Err(req.parts.status.to_string().into())));
        }

        // Utility closure to propagate potential error from conditionally executed trace
        let trace_fn = || {
            trace!(
                body = std::str::from_utf8(&req.body)?,
                "raw JSON event received from Lambda"
            );
            Ok(())
        };
        if let Err(err) = trace_fn() {
            error!(error = ?err, "Failed to parse raw JSON event received from Lambda. The handler will not be called. Log at TRACE level to see the payload.");
            return RuntimeApiResponseFuture::Ready(Some(Err(err)));
        };

        let request_id = req.context.request_id.clone();
        let lambda_event = match deserializer::deserialize::<EventPayload>(req.body, req.context) {
            Ok(lambda_event) => lambda_event,
            Err(err) => match build_event_error_request(&request_id, err) {
                Ok(request) => return RuntimeApiResponseFuture::Ready(Some(Ok(request))),
                Err(err) => {
                    error!(error = ?err, "failed to build error response for Lambda Runtime API");
                    return RuntimeApiResponseFuture::Ready(Some(Err(err)));
                }
            },
        };

        // Once the handler input has been generated successfully, the
        let fut = self.inner.call(lambda_event);
        RuntimeApiResponseFuture::Future(fut, request_id, PhantomData)
    }
}

fn build_event_error_request<T>(request_id: &str, err: T) -> Result<http::Request<Body>, BoxError>
where
    T: Into<Diagnostic> + Debug,
{
    error!(error = ?err, "Request payload deserialization into LambdaEvent<T> failed. The handler will not be called. Log at TRACE level to see the payload.");
    EventErrorRequest::new(request_id, err).into_req()
}

#[pin_project(project = RuntimeApiResponseFutureProj)]
pub enum RuntimeApiResponseFuture<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> {
    Future(
        #[pin] F,
        String,
        PhantomData<(
            (),
            Response,
            BufferedResponse,
            StreamingResponse,
            StreamItem,
            StreamError,
        )>,
    ),
    Ready(Option<Result<http::Request<Body>, BoxError>>),
}

impl<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> Future
    for RuntimeApiResponseFuture<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
where
    F: Future<Output = Result<Response, Diagnostic>>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    type Output = Result<http::Request<Body>, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        task::Poll::Ready(match self.as_mut().project() {
            RuntimeApiResponseFutureProj::Future(fut, request_id, _) => match ready!(fut.poll(cx)) {
                Ok(ok) => EventCompletionRequest::new(request_id, ok).into_req(),
                Err(err) => EventErrorRequest::new(request_id, err).into_req(),
            },
            RuntimeApiResponseFutureProj::Ready(ready) => ready.take().expect("future polled after completion"),
        })
    }
}
