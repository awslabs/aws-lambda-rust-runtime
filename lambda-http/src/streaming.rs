use crate::{http::header::SET_COOKIE, request::LambdaRequest, Request, RequestExt};
use bytes::Bytes;
use core::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
pub use http::{self, Response};
use http_body::Body;
use lambda_runtime::Diagnostic;
pub use lambda_runtime::{Error, LambdaEvent, MetadataPrelude, Service, StreamResponse};
use std::marker::PhantomData;
use tokio_stream::Stream;

/// An adapter that lifts a standard [`Service<Request>`] into a
/// [`Service<LambdaEvent<LambdaRequest>>`] which produces streaming Lambda HTTP
/// responses.
pub struct StreamAdapter<'a, S, B> {
    service: S,
    _phantom_data: PhantomData<&'a B>,
}

impl<'a, S, B, E> From<S> for StreamAdapter<'a, S, B>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    E: Debug + Into<Diagnostic>,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    fn from(service: S) -> Self {
        StreamAdapter {
            service,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, S, B, E> Service<LambdaEvent<LambdaRequest>> for StreamAdapter<'a, S, B>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    B: Body + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
    E: Debug + Into<Diagnostic>,
{
    type Response = StreamResponse<BodyStream<B>>;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, E>> + Send + 'a>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaEvent<LambdaRequest>) -> Self::Future {
        let event: Request = req.payload.into();
        let fut = self.service.call(event.with_lambda_context(req.context));
        Box::pin(async move {
            let res = fut.await?;
            let (parts, body) = res.into_parts();

            let mut headers = parts.headers;
            let cookies = headers
                .get_all(SET_COOKIE)
                .iter()
                .map(|c| String::from_utf8_lossy(c.as_bytes()).to_string())
                .collect::<Vec<_>>();
            headers.remove(SET_COOKIE);

            Ok(StreamResponse {
                metadata_prelude: MetadataPrelude {
                    headers,
                    status_code: parts.status,
                    cookies,
                },
                stream: BodyStream { body },
            })
        })
    }
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
    lambda_runtime::run(StreamAdapter::from(handler)).await
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
