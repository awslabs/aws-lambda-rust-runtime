use crate::{http::header::SET_COOKIE, request::LambdaRequest, tower::ServiceBuilder, Request, RequestExt};
use bytes::Bytes;
pub use http::{self, Response};
use http_body::Body;
use lambda_runtime::Diagnostic;
pub use lambda_runtime::{
    self,
    tower::{
        util::{MapRequest, MapResponse},
        ServiceExt,
    },
    Error, LambdaEvent, MetadataPrelude, Service, StreamResponse,
};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

/// Converts a handler into a streaming-compatible service for use with AWS
/// Lambda.
///
/// This function wraps a `Service` implementation, transforming its input and
/// output to be compatible with AWS Lambda's streaming response feature. It
/// provides the necessary middleware to handle `LambdaEvent` requests and
/// converts the `http::Response` into a `StreamResponse` containing a metadata
/// prelude and body stream.
#[allow(clippy::type_complexity)]
pub fn into_streaming_response<'a, S, B, E>(
    handler: S,
) -> MapResponse<
    MapRequest<S, impl FnMut(LambdaEvent<LambdaRequest>) -> Request>,
    impl FnOnce(Response<B>) -> StreamResponse<BodyStream<B>> + Clone,
>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    E: Debug + Into<Diagnostic>,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    ServiceBuilder::new()
        .map_request(|req: LambdaEvent<LambdaRequest>| {
            let event: Request = req.payload.into();
            event.with_lambda_context(req.context)
        })
        .service(handler)
        .map_response(|res: Response<B>| {
            let (parts, body) = res.into_parts();

            let mut prelude_headers = parts.headers;

            let cookies = prelude_headers
                .get_all(SET_COOKIE)
                .iter()
                .map(|c| String::from_utf8_lossy(c.as_bytes()).to_string())
                .collect::<Vec<String>>();

            prelude_headers.remove(SET_COOKIE);

            let metadata_prelude = MetadataPrelude {
                headers: prelude_headers,
                status_code: parts.status,
                cookies,
            };

            StreamResponse {
                metadata_prelude,
                stream: BodyStream { body },
            }
        })
}

/// Starts the Lambda Rust runtime and stream response back [Configure Lambda
/// Streaming
/// Response](https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html).
///
/// This takes care of transforming the LambdaEvent into a [`Request`] and
/// accepts [`http::Response<http_body::Body>`] as response.
pub async fn run_with_streaming_response<'a, S, B, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    E: Debug + Into<Diagnostic>,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    let svc = into_streaming_response(handler);
    lambda_runtime::run(svc).await
}

pin_project_lite::pin_project! {
pub struct BodyStream<B> {
    #[pin]
    pub(crate) body: B,
}
}

impl<B> Stream for BodyStream<B>
where
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    type Item = Result<B::Data, B::Error>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures_util::ready!(self.as_mut().project().body.poll_frame(cx)?) {
            Some(frame) => match frame.into_data() {
                Ok(data) => Poll::Ready(Some(Ok(data))),
                Err(_frame) => Poll::Ready(None),
            },
            None => Poll::Ready(None),
        }
    }
}
