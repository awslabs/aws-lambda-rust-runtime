use std::fmt::{Debug, Display};
use bytes::{buf::FromBuf, Bytes, IntoBuf};
use failure::Fail;
use futures::{
    future::{result, FutureResult},
    Async, Future, Poll,
};
use http::{Method, Response, Request, Uri, StatusCode};
use hyper::{Body, Client};
use serde_json::Value;
use tower_service::Service;
use tower_util::ServiceFn;
mod client_ext;
use crate::client_ext::*;

pub type LambdaFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

#[derive(Fail, Debug)]
enum RuntimeError {
    #[fail(display = "{}", _0)]
    Http(#[fail(cause)] hyper::error::Error),
    #[fail(display = "{}", _0)]
    Json(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "{}", _0)]
    Utf8Error(#[fail(cause)] std::string::FromUtf8Error),
    #[fail(display = "{}", _0)]
    InvalidUri(#[fail(cause)] http::uri::InvalidUri),
}

trait Handler<Event, Response, Error>
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
    Error: Fail + Display + Debug + Sync + 'static,
    F: FnMut(Event) -> Result<Response, Error>,
{
    fn run(&mut self, event: Event) -> Result<Response, Error> {
        (self)(event)
    }
}

impl<Event, Response, Error> Service<Event> for Handler<Event, Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync,
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

fn main() -> Result<(), RuntimeError> {
    let handler = |event: Bytes| -> Result<Bytes, RuntimeError> {
        let event = String::from_utf8(event.to_vec()).map_err(RuntimeError::Utf8Error)?;
        let value: Value = serde_json::from_str(&event).map_err(RuntimeError::Json)?;
        println!("{:#?}", value);
        Ok(Bytes::from(event))
    };

    let uri = "https://httpbin.org/json"
        .parse::<Uri>()
        .map_err(RuntimeError::InvalidUri)?;

    let mut runtime = Runtime::new(ServiceFn::new(hyper), uri);
    let f = runtime.next_event()
        .and_then(move|event| {
            let body = event.into_body();
            handler(body)
        })
        .then(move|res| match res {
            Ok(bytes) => runtime.ok_response(bytes),
            Err(_) => runtime.err_response(Bytes::from("error!"))
        })
        .map(|res| println!("{}", res.status()))
        .map_err(|e| panic!("{}", e));

    tokio::run(f);
    Ok(())
}

#[derive(Clone)]
struct Runtime<T>
where
    T: Service<Request<Bytes>>
{
    inner: T,
    uri: Uri
}

impl<T> Runtime<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static
{
    pub fn new(inner: T, uri: Uri) -> Self {
        Runtime {
            inner,
            uri
        }
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
        Request::builder()
            .uri(uri)
            .method(method)
            .body(body)
            .unwrap()
    }
}

fn process<Event, Response, Error, Handler>(event: Event, mut handler: Handler) -> Result<Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + 'static,
    Handler: FnMut(Event) -> Result<Response, Error>,
{
    (handler)(event)
}

fn complete(outcome: Result<Bytes, RuntimeError>) -> Result<(), RuntimeError> {
    match outcome {
        Ok(_request_id) => println!("Done!"),
        Err(e) => eprintln!("Lambda function panicked: {}", e),
    }
    Ok(())
}
