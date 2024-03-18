use crate::requests::{EventCompletionRequest, IntoRequest};
use crate::runtime::LambdaInvocation;
use crate::types::Diagnostic;
use crate::{deserializer, IntoFunctionResponse};
use crate::{EventErrorRequest, LambdaEvent};
use futures::{future::BoxFuture, FutureExt, Stream};
use lambda_runtime_api_client::{body::Body, BoxError};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future;
use std::marker::PhantomData;
use std::task;
use tower::Service;
use tracing::{debug, error, trace};

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

impl<'a, S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
    Service<LambdaInvocation>
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
    S: Service<LambdaEvent<EventPayload>, Response = Response, Error = Diagnostic<'a>>,
    S::Future: Send + 'static,
    EventPayload: for<'de> Deserialize<'de>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    type Response = http::Request<Body>;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|err| BoxError::from(format!("{}: {}", err.error_type, err.error_message)))
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        #[cfg(debug_assertions)]
        if req.parts.status.is_server_error() {
            error!("Lambda Runtime server returned an unexpected error");
            return future::ready(Err(req.parts.status.to_string().into())).boxed();
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
            error!(error = ?err, "failed to parse raw JSON event received from Lambda");
            return future::ready(Err(err)).boxed();
        };

        let request_id = req.context.request_id.clone();
        let lambda_event = match deserializer::deserialize::<EventPayload>(&req.body, req.context) {
            Ok(lambda_event) => lambda_event,
            Err(err) => match build_event_error_request(&request_id, err) {
                Ok(request) => return future::ready(Ok(request)).boxed(),
                Err(err) => {
                    error!(error = ?err, "failed to build error response for Lambda Runtime API");
                    return future::ready(Err(err)).boxed();
                }
            },
        };

        // Once the handler input has been generated successfully, the
        let fut = self.inner.call(lambda_event);
        async move {
            match fut.await {
                Ok(result) => EventCompletionRequest::new(&request_id, result).into_req(),
                Err(err) => EventErrorRequest::new(&request_id, err).into_req(),
            }
        }
        .boxed()
    }
}

fn build_event_error_request<'a, T>(request_id: &'a str, err: T) -> Result<http::Request<Body>, BoxError>
where
    T: Into<Diagnostic<'a>> + Debug,
{
    debug!(error = ?err, "building error response for Lambda Runtime API");
    EventErrorRequest::new(request_id, err).into_req()
}
