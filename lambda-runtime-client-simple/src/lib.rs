#[macro_use]
extern crate serde_derive;

use crate::{hyper_tower::*, settings::Config};
use bytes::{buf::FromBuf, Bytes, IntoBuf};
use futures::{
    future::{poll_fn, result, FutureResult},
    Async, Future, Poll,
};
use http::{Method, Request, Response, Uri};
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

pub type Error = Box<std::error::Error + Sync + Send + 'static>;

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

fn run<F>(mut f: impl Handler<Bytes, Bytes> + 'static, catch: F, config: Config) -> Result<(), Error>
where
    F: FnOnce(Error) -> String,
    F: Send + 'static,
{
    let uri = config.endpoint.parse::<Uri>()?;

    let mut runtime = Runtime::new(ServiceFn::new(hyper), uri);
    let f = runtime
        .next_event()
        .and_then(move |event| {
            let body = event.into_body();
            poll_fn(move || {
                blocking(|| f.run(body.clone()))
            }).map_err(|_| panic!("the threadpool shut down"))
        })
        .and_then(move |res| match res {
            Ok(bytes) => runtime.ok_response(bytes),
            Err(e) => runtime.err_response(Bytes::from(catch(e))),
        })
        .map(|_| ())
        .map_err(|e| panic!("{}", e));

    let rt = tokio::runtime::Runtime::new()?;
    let _ = rt.block_on_all(f);
    Ok(())
}

pub fn start<F>(f: impl Handler<Bytes, Bytes> + 'static, catch: F) -> Result<(), Error>
where
    F: FnOnce(Error) -> String,
    F: Send + 'static,
{
    let config = Config::from_env()?;
    run(f, catch, config)
}

#[derive(Clone)]
struct Runtime<T>
where
    T: Service<Request<Bytes>>,
{
    inner: T,
    uri: Uri,
}

impl<T> Runtime<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static,
{
    pub fn new(inner: T, uri: Uri) -> Self {
        Runtime { inner, uri }
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

    pub fn init_failure(&mut self, body: Bytes) -> LambdaFuture<Response<Bytes>, T::Error> {
        let request = self.build(self.uri.clone(), Method::POST, body);
        self.call(request)
    }

    fn call(&mut self, request: Request<Bytes>) -> LambdaFuture<Response<Bytes>, T::Error> {
        let f = self.inner.call(request);
        Box::new(f)
    }

    fn build(&self, uri: Uri, method: Method, body: Bytes) -> Request<Bytes> {
        Request::builder().uri(uri).method(method).body(body).unwrap()
    }
}
