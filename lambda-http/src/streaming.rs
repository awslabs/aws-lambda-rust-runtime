use crate::{http::header::SET_COOKIE, request::LambdaRequest, Request, RequestExt};
use bytes::Bytes;
use core::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{Stream, TryFutureExt};
pub use http::{self, Response};
use http_body::Body;
use lambda_runtime::{
    tower::{
        util::{MapRequest, MapResponse},
        ServiceBuilder, ServiceExt,
    },
    Diagnostic,
};
pub use lambda_runtime::{Error, LambdaEvent, MetadataPrelude, Service, StreamResponse};
use std::{future::Future, marker::PhantomData};

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
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    type Response = StreamResponse<BodyStream<B>>;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'a>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaEvent<LambdaRequest>) -> Self::Future {
        let event: Request = req.payload.into();
        Box::pin(
            self.service
                .call(event.with_lambda_context(req.context))
                .map_ok(into_stream_response),
        )
    }
}

/// Builds a streaming-aware Tower service from a `Service<Request>` **without**
/// boxing its future (no heap allocation / vtable).
///
/// Transforms `LambdaEvent<LambdaRequest>` into `Request` with Lambda context
/// and wraps `Response<B>` into `StreamResponse<BodyStream<B>>`.
///
/// Used internally by [`run_with_streaming_response`]; not part of the public
/// API.
#[allow(clippy::type_complexity)]
fn into_stream_service<'a, S, B, E>(
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
        .map_response(into_stream_response)
}

/// Converts an `http::Response<B>` into a streaming Lambda response.
fn into_stream_response<B>(res: Response<B>) -> StreamResponse<BodyStream<B>>
where
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    let (parts, body) = res.into_parts();

    let mut headers = parts.headers;
    let cookies = headers
        .get_all(SET_COOKIE)
        .iter()
        .map(|c| String::from_utf8_lossy(c.as_bytes()).to_string())
        .collect::<Vec<_>>();
    headers.remove(SET_COOKIE);

    StreamResponse {
        metadata_prelude: MetadataPrelude {
            headers,
            status_code: parts.status,
            cookies,
        },
        stream: BodyStream { body },
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
    lambda_runtime::run(into_stream_service(handler)).await
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

#[cfg(test)]
mod test_stream_adapter {
    use super::*;

    use crate::Body;
    use http::StatusCode;

    // A middleware that logs requests before forwarding them to another service
    struct LogService<S> {
        inner: S,
    }

    impl<S> Service<LambdaEvent<LambdaRequest>> for LogService<S>
    where
        S: Service<LambdaEvent<LambdaRequest>>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = S::Future;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, event: LambdaEvent<LambdaRequest>) -> Self::Future {
            println!("Lambda event: {event:#?}");
            self.inner.call(event)
        }
    }

    #[test]
    fn stream_adapter_is_boxable() {
        // Works with a concrete service stack (no boxing)
        let svc = ServiceBuilder::new()
            .layer_fn(|service| LogService { inner: service })
            .layer_fn(StreamAdapter::from)
            .service_fn(
                |_req: Request| async move { http::Response::builder().status(StatusCode::OK).body(Body::Empty) },
            );
        // Also works when the stack is boxed (type-erased)
        let _boxed_svc = svc.boxed();
    }
}
