#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tokio_trace;
extern crate tokio_trace_fmt;

use crate::settings::Config;
use futures::lazy;
use hyper::client::HttpConnector;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_trace::field;

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

fn run<A>(handler: A, config: Config)
where
    A: Fn(Body) -> FutureObj<Bytes, Error> + Send + 'static,
{
    let uri = config.endpoint.parse::<Uri>().unwrap();
    let listener = EventStream::new(uri.clone());
    let svc = listener
        // .next_event()
        // .and_then(move |event| {
        .map_err(|e| info!({ error = field::debug(e) }, "error accepting event"))
        .for_each(move |event| {
            dbg!("Accepted event");
            let (tx, rx) = sink(uri.clone());
            let handle = (handler)(event.into_body())
                .then(move |res| {
                    dbg!(&res);
                    tx.send(res).map_err(|e| {
                        dbg!("Unable to send");
                    })
                })
                .map(|sink| {
                    dbg!("Responded to the lambda runtime");
                });
            tokio::spawn(handle);
            Ok(())
        });

    tokio::run(lazy(move || svc));
}

fn sink(uri: Uri) -> (Sender<Result<Bytes, Error>>, EventSink) {
    let (tx, rx) = mpsc::channel(1);
    (tx, EventSink::new(rx, uri))
}

pub fn start<A>(f: A) -> Result<(), Error>
where
    A: Fn(Body) -> FutureObj<Bytes, Error> + Send + 'static,
{
    let subscriber = tokio_trace_fmt::FmtSubscriber::builder().full().finish();
    tokio_trace::subscriber::with_default(subscriber, || {
        span!("lambda").enter(|| {
            info!("Reading config");
            let config = Config::from_env();
            info!("read config; launching function");
            run(f, config);
        });
    });

    Ok(())
}

struct EventStream {
    uri: Uri,
}

impl EventStream {
    fn new(uri: Uri) -> Self {
        let mut parts = uri.clone().into_parts();
        parts.scheme = Some("http".parse().unwrap());
        parts.path_and_query = Some("/2018-06-01/runtime/invocation/next".parse().unwrap());
        let uri = Uri::from_parts(parts).unwrap();
        info!({ uri = field::debug(&uri) }, "parsed endpoint");
        Self { uri }
    }

    fn next_event(&mut self) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
        let client = Client::new();
        span!("next_event").enter(|| client.get(self.uri.clone()))
    }
}

impl Stream for EventStream {
    type Item = Response<Body>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.next_event().poll()? {
                Async::Ready(res) => return dbg!(Ok(Async::Ready(Some(res)))),
                Async::NotReady => return dbg!(Ok(Async::NotReady)),
            }
        }
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
            .map_err(|e| {
                dbg!(e);
                ()
            })
            .for_each(move |res| {
                dbg!(&res);
                c.complete_event(res)
            })
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
        dbg!(&req);
        self.call(req)
            .map(|t| {
                dbg!(t);
            })
            .map_err(|e| {
                dbg!(e);
            })
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
    let mut parts = uri.clone().into_parts();
    parts.scheme = Some("http".parse().unwrap());
    parts.path_and_query = Some(
        "/2018-06-01/runtime/invocation/52fdfc07-2182-154f-163f-5f0f9a621d72/response"
            .parse()
            .unwrap(),
    );
    let uri = Uri::from_parts(parts).unwrap();

    Request::builder()
        .uri(uri)
        .method(method)
        .body(Body::empty())
        .unwrap()
}
