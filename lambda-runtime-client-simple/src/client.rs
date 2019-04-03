use bytes::Bytes;
use futures::{Async, Future};
use hyper::{Request, Uri};
use std::{marker::PhantomData, time::Duration};
use tower::{builder::ServiceBuilder, util::ServiceExt, Service};
use tower_add_origin::{AddOrigin, Builder as AddOriginBuilder};
use tower_buffer::{Buffer, BufferLayer};
use tower_hyper::retry::RetryPolicy;
use tower_in_flight_limit::{InFlightLimit, InFlightLimitLayer};
use tower_retry::RetryLayer;
use tower_timeout::{Timeout, TimeoutLayer};

type Err = Box<dyn std::error::Error + Send + Sync>;

pub(crate) struct LambdaSvc<S, B>
where
    S: Service<Request<B>> + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    inner: Buffer<InFlightLimit<Timeout<AddOrigin<S>>>, Request<B>>,
    _phan: PhantomData<S>,
}

impl<S, B> LambdaSvc<S, B>
where
    S: Service<Request<B>> + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    fn new(svc: S, origin: Uri) -> Self {
        let svc: AddOrigin<S> = AddOriginBuilder::new().uri(origin).build(svc).unwrap();
        let svc = ServiceBuilder::new()
            .layer(InFlightLimitLayer::new(1))
            .layer(TimeoutLayer::new(Duration::from_millis(300)))
            .build_service(svc)
            .unwrap();

        Self {
            inner: Buffer::new(svc, 1).unwrap(),
            _phan: PhantomData,
        }
    }
}

impl<S, B> Service<Request<B>> for LambdaSvc<S, B>
where
    S: Service<Request<B>> + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = Err;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        self.inner.poll_ready().map_err(|e| e.into())
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let fut = self.inner.call(req);
        Box::new(fut)
    }
}
