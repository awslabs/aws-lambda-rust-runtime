#[macro_use]
extern crate serde_derive;

use crate::{hyper_tower::*, settings::Config};
use bytes::{buf::FromBuf, Bytes, IntoBuf};
use futures::{try_ready, Async, Future, Poll, Stream};
use http::{Method, Request, Response, Uri};
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_service::Service;
use tower_util::ServiceFn;

pub mod hyper_tower;
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

struct RuntimeLoop {
    cfg: Arc<Config>,
}

impl RuntimeLoop {
    fn new(cfg: Config) -> Self {
        let cfg = Arc::new(cfg);
        Self { cfg }
    }

    fn run<A>(&self, mut handler: A)
    where
        A: Handler<Bytes, Bytes> + Clone + 'static,
    {
        let uri = self.cfg.endpoint.parse::<Uri>().unwrap();
        let runtime = RuntimeClient::new(ServiceFn::new(hyper), uri.clone());
        let listener = EventListener::new(runtime, uri.clone());
        let (tx, rx) = oneshot::channel::<Result<Bytes, Error>>();

        let svc = listener
            .for_each(move |event| {
                let body = event.into_body();
                let uri = uri.clone();
                let fut = handler
                    .run(body)
                    .then(move |res| {
                        let mut rt = RuntimeClient::new(ServiceFn::new(hyper), uri);
                        rt.complete_event(res)
                    })
                    .map(|_| ())
                    .map_err(|_| ());

                tokio::spawn(fut);
                futures::future::ok(())
            })
            .map_err(|e| panic!("Error: {}", e));

        tokio::run(svc);
    }
}

pub fn start<A>(f: A) -> Result<(), Error>
where
    A: Handler<Bytes, Bytes> + Clone + 'static,
{
    let config = Config::from_env()?;
    let runtime = RuntimeLoop::new(config);

    runtime.run(f.clone());
    Ok(())
}

struct EventListener<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static,
{
    inner: RuntimeClient<T>,
    uri: Uri,
}

impl<T> EventListener<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static,
{
    fn new(inner: RuntimeClient<T>, uri: Uri) -> Self {
        Self { inner, uri }
    }

    fn next_event(&mut self) -> FutureObj<Response<Bytes>, T::Error> {
        let request = make_req(self.uri.clone(), Method::GET, Bytes::new());
        self.inner.call(request)
    }
}

impl<T> Stream for EventListener<T>
where
    T: Service<Request<Bytes>, Response = Response<Bytes>>,
    T::Future: Send + 'static,
{
    type Item = Response<Bytes>;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<Response<Bytes>>, Self::Error> {
        let value: Response<Bytes> = try_ready!(self.next_event().poll());
        Ok(Async::Ready(Some(value)))
    }
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

    pub fn complete_event(
        &mut self,
        res: Result<Bytes, T::Error>,
    ) -> FutureObj<Response<Bytes>, T::Error> {
        let req = match res {
            Ok(body) => make_req(self.uri.clone(), Method::POST, body),
            Err(e) => panic!("Error"),
        };
        self.call(req)
    }

    fn call(&mut self, request: Request<Bytes>) -> FutureObj<Response<Bytes>, T::Error> {
        Box::new(self.inner.call(request))
    }
}

fn make_req(uri: Uri, method: Method, body: Bytes) -> Request<Bytes> {
    Request::builder()
        .uri(uri)
        .method(method)
        .body(body)
        .unwrap()
}
