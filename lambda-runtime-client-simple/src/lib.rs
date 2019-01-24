#[macro_use]
extern crate serde_derive;

use bytes::{buf::FromBuf, Bytes, IntoBuf};
use failure::Fail;
use futures::{
    future::{result, FutureResult},
    Async, Future, Poll,
};
use http::{Method, Request, Response, Uri};
use serde_json::Value;
use std::fmt::{Debug, Display};
use tower_service::Service;
use tower_util::ServiceFn;
use crate::{hyper_tower::*, settings::Config};

pub mod hyper_tower;
pub mod settings;

pub type LambdaFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

#[macro_export]
macro_rules! lambda {
    ($handler:ident, on_err = $catch:ident) => {
        $crate::start($handler, $catch)
    };
}

#[derive(Fail, Debug)]
pub enum RuntimeError {
    #[fail(display = "{}", _0)]
    Http(#[fail(cause)] hyper::error::Error),
    #[fail(display = "{}", _0)]
    Json(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "{}", _0)]
    Utf8Error(#[fail(cause)] std::string::FromUtf8Error),
    #[fail(display = "{}", _0)]
    EnvError(#[fail(cause)] envy::Error),
    #[fail(display = "{}", _0)]
    InvalidUri(#[fail(cause)] http::uri::InvalidUri),
}

pub trait Handler<Event, Response, Error>: Send
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + 'static,
{
    fn run(&mut self, event: Event) -> Result<Response, Error>;
}

impl<Event, Response, Error, F> Handler<Event, Response, Error> for F
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + Send + 'static,
    F: FnMut(Event) -> Result<Response, Error> + Send,
{
    fn run(&mut self, event: Event) -> Result<Response, Error> {
        (self)(event)
    }
}

impl<Event, Response, Error> Service<Event> for Handler<Event, Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + Send + 'static,
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

fn run<F>(
    mut f: impl Handler<Bytes, Bytes, RuntimeError> + 'static,
    catch: F,
    config: Config,
) -> Result<(), RuntimeError>
where
    F: FnOnce(RuntimeError) -> String,
    F: Send + 'static,
{
    let uri = config.endpoint.parse::<Uri>().map_err(RuntimeError::InvalidUri)?;

    let mut runtime = Runtime::new(ServiceFn::new(hyper), uri);
    let f = runtime
        .next_event()
        .and_then(move |event| {
            let body = event.into_body();
            f.run(body)
        })
        .then(move |res| match res {
            Ok(bytes) => runtime.ok_response(bytes),
            Err(e) => runtime.err_response(Bytes::from(catch(e))),
        })
        .map(|_| ())
        .map_err(|e| panic!("{}", e));

    tokio::run(f);
    Ok(())
}

pub fn start<F>(
    f: impl Handler<Bytes, Bytes, RuntimeError> + 'static,
    catch: F,
) -> Result<(), RuntimeError>
where
    F: FnOnce(RuntimeError) -> String,
    F: Send + 'static,
{
    let config = Config::from_env().map_err(RuntimeError::EnvError)?;
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
