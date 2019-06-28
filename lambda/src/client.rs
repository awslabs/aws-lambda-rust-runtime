use crate::{err_fmt, Error};
use bytes::Bytes;
use futures::{
    future::BoxFuture,
    prelude::*,
    task::{Context, Poll},
};
use http::{Method, Request, Response, Uri};
use hyper::Body;
use std::pin::Pin;

#[derive(Debug)]
pub(crate) struct Client {
    base: Uri,
    client: hyper::Client<hyper::client::HttpConnector>,
}

impl Client {
    pub(crate) fn new(uri: Uri) -> Self {
        Self {
            base: uri,
            client: hyper::Client::new(),
        }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Error> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self
                .base
                .scheme_part()
                .ok_or(err_fmt!("PathAndQuery not found"))?;
            let authority = self
                .base
                .authority_part()
                .ok_or(err_fmt!("PathAndQuery not found"))?;
            (scheme, authority)
        };
        let path = parts
            .uri
            .path_and_query()
            .ok_or(err_fmt!("PathAndQuery not found"))?;

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build()?;

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }
}

/// A trait modeling interactions with the [Lambda Runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
pub(crate) trait EventClient<'a>: Send + Sync {
    /// A future containing the next event from the Lambda Runtime API.
    type Fut: Future<Output = Result<Response<Bytes>, Error>> + Send + 'a;
    fn call(&self, req: Request<Bytes>) -> Self::Fut;
}

impl<'a> EventClient<'a> for Client {
    type Fut = BoxFuture<'a, Result<Response<Bytes>, Error>>;

    fn call(&self, req: Request<Bytes>) -> Self::Fut {
        use futures::compat::{Future01CompatExt, Stream01CompatExt};
        use pin_utils::pin_mut;

        let req = {
            let (parts, body) = req.into_parts();
            let body = Body::from(body);
            Request::from_parts(parts, body)
        };
        let req = self.set_origin(req).unwrap();
        let res = self.client.request(req).compat();
        let fut = async {
            let res = res.await?;
            let (parts, body) = res.into_parts();
            let body = body.compat();
            pin_mut!(body);

            let mut buf: Vec<u8> = vec![];
            while let Some(Ok(chunk)) = body.next().await {
                let mut chunk: Vec<u8> = chunk.into_bytes().to_vec();
                buf.append(&mut chunk)
            }
            let buf = Bytes::from(buf);
            let res = Response::from_parts(parts, buf);
            Ok(res)
        };

        fut.boxed()
    }
}

/// The `Stream` implementation for `EventStream` converts a `Future`
/// containing the next event from the Lambda Runtime into a continuous
/// stream of events. While _this_ stream will continue to produce
/// events indefinitely, AWS Lambda will only run the Lambda function attached
/// to this runtime *if and only if* there is an event available for it to process.
/// For Lambda functions that receive a “warm wakeup”—i.e., the function is
/// readily available in the Lambda service's cache—this runtime is able
/// to immediately fetch the next event.
pub(crate) struct EventStream<'a, T>
where
    T: EventClient<'a>,
{
    current: Option<BoxFuture<'a, Result<Response<Bytes>, Error>>>,
    client: &'a T,
}

impl<'a, T> EventStream<'a, T>
where
    T: EventClient<'a>,
{
    pub(crate) fn new(inner: &'a T) -> Self {
        Self {
            current: None,
            client: inner,
        }
    }

    pub(crate) fn next_event(&self) -> BoxFuture<'a, Result<Response<Bytes>, Error>> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_static("/runtime/invocation/next"))
            .body(Bytes::new())
            .unwrap();
        Box::pin(self.client.call(req))
    }
}

#[must_use = "streams do nothing unless you `.await` or poll them"]
impl<'a, T> Stream for EventStream<'a, T>
where
    T: EventClient<'a>,
{
    type Item = Result<Response<Bytes>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // The `loop` is used to drive the inner future (`current`) to completion, advancing
        // the state of this stream to yield a new `Item`. Loops like the one below are
        // common in many hand-implemented `Futures` and `Streams`.
        loop {
            // The stream first checks an inner future is set. If the future is present,
            // a runtime polls the inner future to completion.
            if let Some(current) = &mut self.current {
                match current.as_mut().poll(cx) {
                    // If the inner future signals readiness, we:
                    // 1. Create a new Future that represents the _next_ event which will be polled
                    // by subsequent iterations of this loop.
                    // 2. Return the current future, yielding the resolved future.
                    Poll::Ready(res) => {
                        let next = self.next_event();
                        self.current = Some(Box::pin(next));
                        return Poll::Ready(Some(res));
                    }
                    // Otherwise, the future signals that it's not ready, so we propagate the
                    // Poll::Pending signal to the caller.
                    Poll::Pending => return Poll::Pending,
                }
            } else {
                self.current = Some(self.next_event());
            }
        }
    }
}
