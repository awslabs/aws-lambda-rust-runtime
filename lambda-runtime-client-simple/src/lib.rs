#[macro_use]
extern crate serde_derive;

use crate::{hyper_tower::*, settings::Config};
use bytes::{buf::FromBuf, Bytes, IntoBuf};
use futures::{
    future::{loop_fn, poll_fn, result, FutureResult, Loop},
    Async, Future, Poll,
};
use http::{Method, Request, Response, Uri};
use std::sync::Arc;
use tokio_threadpool::blocking;
use tower_service::Service;
use tower_util::ServiceFn;

pub mod hyper_tower;
pub mod settings;

pub type LambdaFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

#[macro_export]
macro_rules! lambda {
    ($handler:ident, on_err = $catch:ident) => {
        $crate::start($handler, $catch)
    };
}

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

pub trait Handler<Event, Response>: Send
where
    Event: FromBuf,
    Response: IntoBuf,
{
    fn run(&mut self, event: Event) -> Result<Response, Error>;
}

impl<Event, Response, F> Handler<Event, Response> for F
where
    Event: FromBuf,
    Response: IntoBuf,
    F: FnMut(Event) -> Result<Response, Error> + Send,
{
    fn run(&mut self, event: Event) -> Result<Response, Error> {
        (self)(event)
    }
}

impl<Event, Response> Service<Event> for Handler<Event, Response>
where
    Event: FromBuf,
    Response: IntoBuf,
{
    type Response = Response;
    type Error = Error;
    type Future = FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Event) -> Self::Future {
        result(self.run(req))
    }
}

struct RuntimeLoop {
    cfg: Arc<Config>,
}

impl RuntimeLoop {
    fn new(cfg: Config) -> Self {
        let cfg = Arc::new(cfg);
        Self { cfg }
    }

    fn run<A, B>(&self, mut handler: A, catch: B) -> impl Future<Item = (), Error = Error>
    where
        A: Handler<Bytes, Bytes> + 'static,
        B: FnOnce(Error) -> String + Send + 'static,
    {
        let uri = self.cfg.endpoint.parse::<Uri>().unwrap();
        let mut runtime = RuntimeClient::new(ServiceFn::new(hyper), uri);

        runtime
            .next_event()
            .and_then(move |event| {
                let body = event.into_body();
                poll_fn(move || {
                    blocking(|| handler.run(body.clone()))
                        .map_err(|e| panic!("the threadpool shut down: {}", e))
                })
            })
            .and_then(move |res| match res {
                Ok(bytes) => runtime.ok_response(bytes),
                Err(e) => runtime.err_response(Bytes::from(catch(e))),
            })
            .map(|_| ())
            .map_err(|e| panic!("{}", e))
    }
}

pub fn start<A, B>(f: A, catch: B) -> Result<(), Error>
where
    A: Handler<Bytes, Bytes> + Clone + 'static,
    B: FnOnce(Error) -> String + Clone + Send + 'static,
{
    let config = Config::from_env()?;
    let runtime = RuntimeLoop::new(config);

    let lambda = loop_fn(0, move |counter| {
        runtime
            .run(f.clone(), catch.clone())
            .then(move |res| -> Result<_, Error> {
                if res.is_ok() {
                    Ok(Loop::Continue(counter))
                } else {
                    Ok(Loop::Break(counter))
                }
            })
    });
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on_all(lambda);

    Ok(())
}

#[derive(Clone)]
struct RuntimeClient<T>
where
    T: Service<Request<Bytes>>,
{
    inner: T,
    uri: Uri,
}

impl<T> RuntimeClient<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static,
{
    pub fn new(inner: T, uri: Uri) -> Self {
        RuntimeClient { inner, uri }
    }

    pub fn next_event(&mut self) -> LambdaFuture<Response<Bytes>, T::Error> {
        let request = self.build(self.uri.clone(), Method::GET, Bytes::new());
        self.call(request)
    }

    pub fn ok_response(&mut self, body: Bytes) -> LambdaFuture<Response<Bytes>, T::Error> {
        let request = self.build(self.uri.clone(), Method::POST, body);
        self.call(request)
    }

    pub fn err_response(&mut self, body: Bytes) -> LambdaFuture<Response<Bytes>, T::Error> {
        let request = self.build(self.uri.clone(), Method::POST, body);
        self.call(request)
    }

    fn call(&mut self, request: Request<Bytes>) -> LambdaFuture<Response<Bytes>, T::Error> {
        let f = self.inner.call(request);
        Box::new(f)
    }

    fn build(&self, uri: Uri, method: Method, body: Bytes) -> Request<Bytes> {
        Request::builder()
            .uri(uri)
            .method(method)
            .body(body)
            .unwrap()
    }
}
