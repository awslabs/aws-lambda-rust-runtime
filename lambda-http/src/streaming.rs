use crate::{http::header::SET_COOKIE, request::LambdaRequest, tower::ServiceBuilder, Request, RequestExt};
use bytes::Bytes;
pub use http::{self, Response};
use http_body::Body;
pub use lambda_runtime::{
    self,
    tower::{
        util::{MapRequest, MapResponse},
        ServiceExt,
    },
    Error, LambdaEvent, MetadataPrelude, Service, StreamResponse,
};
use lambda_runtime::{tower::util::BoxService, Diagnostic};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

/// Runs the Lambda runtime with a handler that returns **streaming** HTTP
/// responses.
pub fn into_streaming_response<'a, S, B, E>(
    handler: S,
) -> BoxService<LambdaEvent<LambdaRequest>, StreamResponse<BodyStream<B>>, E>
where
    S: Service<Request, Response = Response<B>, Error = E> + Send + 'static,
    S::Future: Send + 'a,
    E: Debug + Into<Diagnostic> + 'static,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    into_streaming_response_inner::<S, B, E>(handler).boxed()
}

#[allow(clippy::type_complexity)]
fn into_streaming_response_inner<'a, S, B, E>(
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

/// Runs the Lambda runtime with a handler that returns **streaming** HTTP
/// responses.
///
/// See the [AWS docs for response streaming].
///
/// [AWS docs for response streaming]:
///     https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html
pub async fn run_with_streaming_response<'a, S, B, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    E: Debug + Into<Diagnostic>,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    let svc = into_streaming_response_inner(handler);
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
