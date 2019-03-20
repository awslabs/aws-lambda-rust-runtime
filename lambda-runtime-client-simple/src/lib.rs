#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tokio_trace;
extern crate tokio_trace_fmt;

use crate::{client::LambdaSvc, settings::Config};
use hyper::client::HttpConnector;
use std::{str::FromStr, sync::Arc};
use tokio_trace::field;

use bytes::{buf::FromBuf, Bytes, IntoBuf};
use futures::{Async, Future, Poll, Stream};
use http::{
    uri::{Parts, PathAndQuery, Uri},
    Request, Response,
};
use hyper::{Body, Client};
use tokio_trace_fmt::FmtSubscriber;
use tower_service::Service;

pub mod client;
#[cfg(test)]
mod mock;
pub mod settings;

#[macro_export]
/// The helper lambda macro.
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
    let client = RuntimeClient::new(uri);
    let listener = EventStream::new(client.clone());
    // Since `EventStream` implements Stream, the `.for_each` combinator is used to
    // process each incoming event.
    let svc = listener
        .map_err(|e| error!({ error = field::debug(e) }, "error accepting event"))
        .for_each(move |event: Response<Body>| {
            let client = client.clone();
            info!({ event = field::debug(&event) }, "Received event");
            let handle = (handler)(event.into_body())
                .then(move |res: Result<Bytes, Error>| {
                    info!({ event = field::debug(&res) }, "processed event");

                    client.complete_event(res).map_err(|e| {
                        error!({ error = field::debug(e) }, "Unable to send response");
                    })
                })
                .map(|resp: Response<Body>| {
                    info!("Responded to the lambda runtime");
                });
            // `tokio::spawn` spawns `handle` onto Tokio's event loop, allowing the above future to execute.
            tokio::spawn(handle);
            Ok(())
        });

    tokio::run(svc);
}

pub fn start<A>(f: A) -> Result<(), Error>
where
    A: Fn(Body) -> FutureObj<Bytes, Error> + Send + 'static,
{
    let subscriber = FmtSubscriber::builder().full().finish();
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

pub struct EventStream {
    client: RuntimeClient,
    current: Option<FutureObj<Response<Body>, Error>>,
}

impl EventStream {
    fn new(client: RuntimeClient) -> Self {
        Self {
            client: client,
            current: None,
        }
    }

    fn next_event(&self) -> FutureObj<Response<Body>, Error> {
        span!("next_event").enter(|| {
            let fut = self.client.next_event();
            Box::new(fut.map_err(|e| e.into()))
        })
    }
}

/// The `Stream` implementation for `EventStream` converts a `Future`
/// containing the next event from the Lambda Runtime into a continous
/// stream of events. While _this_ stream will continue to produce
/// events indefinitely, AWS Lambda will only run the Lambda function attached
/// to this runtime *if and only if* there is an event availible for it to process.
/// For Lambda functions that receive a “warm wakeup”—i.e., the function is
/// readily availible in the Lambda service's cache—this runtime is able
/// to immediately fetch the next event.
impl Stream for EventStream {
    type Item = Response<Body>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // The `loop` is used to drive the inner future (`current`) to completion, advancing
        // the state of this stream to yield a new `Item`. Loops like the one below are
        // common in many hand-implemented `Futures` and `Streams`.
        loop {
            // The stream first checks an inner future is set. If the future is present,
            // the _Tokio_ runtime polls the inner future to completition. For more details
            // on polling asychronous values, refer to the [documentation](https://docs.rs/futures/0.1/futures/future/trait.Future.html#tymethod.poll) on the poll method on `Future`.
            if self.current.is_some() {
                match self.current.poll()? {
                    // If the current inner future signals readiness/that it is complete:
                    // 1. Create a new Future that represents the _next_ event which will be polled
                    // by subsequent iterations of this loop.
                    // 2. Return the current future, yielding the resolved future.
                    Async::Ready(res) => {
                        self.current = Some(self.next_event());
                        return Ok(Async::Ready(res));
                    }
                    // Otherwise, the future signals that it's not ready, so we do the same
                    // to the Tokio runtime.
                    Async::NotReady => return Ok(Async::NotReady),
                }
            // If the future is not set (due to a cold start from inactivity or a fresh deployment),
            // we set a new Future to be polled in subsequent loops.
            } else {
                self.current = Some(self.next_event());
            }
        }
    }
}

#[derive(Clone)]
pub struct Endpoint {
    base: Uri,
}

impl Endpoint {
    pub fn new(base: Uri) -> Arc<Self> {
        let e = Self { base };
        Arc::new(e)
    }

    pub fn next_event(&self) -> Result<Uri, Error> {
        let mut parts: Parts = self.base.clone().into_parts();

        let query = PathAndQuery::from_str("/2018-06-01/runtime/invocation/next")?;
        parts.path_and_query = Some(query);

        Ok(Uri::from_parts(parts)?)
    }

    pub fn init_error(&self) -> Result<Uri, Error> {
        let mut parts: Parts = self.base.clone().into_parts();

        let query = PathAndQuery::from_str("/2018-06-01/runtime/init/error")?;
        parts.path_and_query = Some(query);

        Ok(Uri::from_parts(parts)?)
    }

    pub fn ok_response(&self, event_id: String) -> Result<Uri, Error> {
        let mut parts: Parts = self.base.clone().into_parts();

        let query = format!("/2018-06-01/runtime/invocation/{}/response", event_id);
        let query = PathAndQuery::from_str(&query)?;
        parts.path_and_query = Some(query);

        Ok(Uri::from_parts(parts)?)
    }

    pub fn err_response(&self, event_id: String) -> Result<Uri, Error> {
        let mut parts: Parts = self.base.clone().into_parts();

        let query = format!("/2018-06-01/runtime/invocation/{}/error", event_id);
        let query = PathAndQuery::from_str(&query)?;
        parts.path_and_query = Some(query);

        Ok(Uri::from_parts(parts)?)
    }
}

#[derive(Clone)]
struct RuntimeClient {
    endpoint: Arc<Endpoint>,
    inner: Client<HttpConnector>,
}

impl RuntimeClient {
    pub fn new(base: Uri) -> Self {
        Self {
            endpoint: Endpoint::new(base),
            inner: Client::new(),
        }
    }

    pub fn next_event(&self) -> FutureObj<Response<Body>, Error> {
        let uri = match self.endpoint.next_event() {
            Ok(uri) => uri,
            Err(e) => return Box::new(futures::future::err(e)),
        };

        Box::new(self.inner.get(uri).map_err(|e| e.into()))
    }

    pub fn complete_event(&self, res: Result<Bytes, Error>) -> FutureObj<Response<Body>, Error> {
        unimplemented!()
    }
}

// Tower's Service is a pleasant abstraction over Futures, clients, and services.
impl Service<Request<Body>> for RuntimeClient {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error> + Send>;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::new(self.inner.request(req).map_err(|e| e.into()))
    }
}
