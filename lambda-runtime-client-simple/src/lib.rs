#[macro_use]
extern crate serde_derive;

use crate::settings::Config;
use hyper::client::HttpConnector;
use tokio::sync::mpsc::{Receiver, Sender};

use bytes::{buf::FromBuf, Bytes, IntoBuf};
use futures::{try_ready, Async, Future, Poll, Sink, Stream};
use http::{Method, Request, Response, Uri};
use hyper::{Body, Client};
use tokio::sync::mpsc;
use tower_service::Service;

pub mod settings;

#[macro_export]
macro_rules! lambda {
    ($handler:ident) => {
        $crate::start($handler)
    };
}

pub type FutureObj<T, E> = Box<Future<Item = T, Error = E> + Send>;
pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

pub trait Handler<Event, Response>: Send
where
    Event: FromBuf,
    Response: IntoBuf,
{
    fn run(&mut self, event: Event) -> FutureObj<Response, Error>;
}

impl<Event, Response, F> Handler<Event, Response> for F
where
    Event: FromBuf,
    Response: IntoBuf,
    F: FnMut(Event) -> FutureObj<Response, Error> + Send,
{
    fn run(&mut self, event: Event) -> FutureObj<Response, Error> {
        (self)(event)
    }
}

impl<Event, Response: 'static> Service<Event> for Handler<Event, Response>
where
    Event: FromBuf,
    Response: IntoBuf,
{
    type Response = Response;
    type Error = Error;
    type Future = FutureObj<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Event) -> Self::Future {
        Box::new(self.run(req))
    }
}

fn run<A>(handler: A, uri: Uri)
where
    A: Fn(Body) -> FutureObj<Bytes, Error> + Send + 'static,
{
    let listener = EventStream::new(uri.clone());
    let svc = listener
        .map_err(|e| println!("error accepting event; error = {}", e))
        .for_each(move |event| {
            let (tx, rx) = sink(uri.clone());

            let handle = (handler)(event.into_body())
                .then(move |res| tx.send(res).map_err(|e| println!("Unable to send = {}", e)))
                .map(|sink| {
                    println!("Responded to the lambda runtime");
                });
            tokio::spawn(handle);
            rx.process()
        });

    tokio::run(svc);
}

fn sink(uri: Uri) -> (Sender<Result<Bytes, Error>>, EventSink) {
    let (tx, rx) = mpsc::channel(1);
    (tx, EventSink::new(rx, uri))
}

pub fn start<A>(f: A) -> Result<(), Error>
where
    A: Fn(Body) -> FutureObj<Bytes, Error> + Send + 'static,
{
    let config = Config::from_env()?;
    let uri = config.endpoint.parse::<Uri>()?;
    run(f, uri);

    Ok(())
}

struct EventStream {
    uri: Uri,
}

impl EventStream {
    fn new(uri: Uri) -> Self {
        Self { uri: uri.clone() }
    }

    fn next_event(&mut self) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
        let client = Client::new();
        client.get(self.uri.clone())
    }
}

impl Stream for EventStream {
    type Item = Response<Body>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let value: Response<Body> = try_ready!(self.next_event().poll());
        Ok(Async::Ready(Some(value)))
    }
}

struct EventSink {
    rx: Receiver<Result<Bytes, Error>>,
    uri: Uri,
}

impl EventSink {
    fn new(rx: Receiver<Result<Bytes, Error>>, uri: Uri) -> Self {
        Self { rx, uri }
    }

    fn process(self) -> impl Future<Item = (), Error = ()> {
        let mut c = RuntimeClient::new(self.uri.clone());
        self.rx
            .map_err(|_| ())
            .for_each(move |res| c.complete_event(res))
    }
}

#[derive(Clone)]
struct RuntimeClient {
    uri: Uri,
    inner: Client<HttpConnector>,
}

impl RuntimeClient {
    pub fn new(uri: Uri) -> Self {
        Self {
            uri,
            inner: Client::new(),
        }
    }

    pub fn complete_event(
        &mut self,
        res: Result<Bytes, Error>,
    ) -> impl Future<Item = (), Error = ()> {
        let req = match res {
            Ok(body) => make_req(self.uri.clone(), Method::POST, body),
            Err(e) => panic!("Error"),
        };
        self.call(req).map(|_| ()).map_err(|_| ())
    }
}

impl Service<Request<Body>> for RuntimeClient {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error> + Send>;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::new(self.inner.request(req))
    }
}

fn make_req(uri: Uri, method: Method, body: Bytes) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(method)
        .body(Body::from(body))
        .unwrap()
}
