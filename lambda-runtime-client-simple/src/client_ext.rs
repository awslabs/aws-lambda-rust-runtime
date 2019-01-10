use futures::{Async, Future, Poll};
use futures::future;
use http::{Request, Response, Method};
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use tower_retry::{Policy, Retry};
use futures::stream::Stream;
use tower_service::Service;
use bytes::Bytes;
use crate::RuntimeError;

#[derive(Clone)]
pub(crate) struct ClientWrapper {
    inner: Client<HttpConnector>,
}

impl ClientWrapper {
    pub fn new() -> Self {
        ClientWrapper {
            inner: Client::new()
        }
    }
}

pub(crate) struct FutureObj<T, E> {
    inner: Box<Future<Item=T, Error=E> + Send>,
}

impl<T, E> Future for FutureObj<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

impl Service<Request<Body>> for ClientWrapper {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = FutureObj<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let f = self.inner.request(req);
        FutureObj { inner: Box::new(f) }
    }
}

#[derive(Clone)]
struct RetryPolicy {
    attempts: u8,
}

impl RetryPolicy {
    fn new(attempts: u8) -> Self {
        RetryPolicy {
            attempts
        }
    }
}

impl Policy<Request<Body>, Response<Body>, hyper::Error> for RetryPolicy {
    type Future = future::FutureResult<Self, ()>;

    fn retry(&self, _: &Request<Body>, result: Result<&Response<Body>, &hyper::Error>) -> Option<Self::Future> {
        if self.attempts == 0 {
            // We ran out of retries, let's abandon.
            return None;
        }

        match result {
            Ok(res) => if res.status().is_server_error() {
                let policy = RetryPolicy::new(self.attempts - 1);
                Some(future::ok(policy))
            } else {
                // 2xx-4xx shouldn't be retried.
                None
            },
            Err(_) => {
                Some(future::ok(RetryPolicy { attempts: self.attempts - 1 }))
            }
        }
    }

    fn clone_request(&self, req: &Request<Body>) -> Option<Request<Body>> {
        if req.method() == http::Method::GET {
            let mut clone = http::Request::new(Body::empty());
            *clone.uri_mut() = req.uri().clone();
            *clone.headers_mut() = req.headers().clone();
            Some(clone)
        } else {
            None
        }
    }
}

pub(crate) fn hyper(req: Request<Bytes>) -> impl Future<Item = Response<Bytes>, Error = RuntimeError> {
    let client = ClientWrapper::new();
    let policy = RetryPolicy::new(3);
    let mut svc = Retry::new(policy, client);

    svc
        .call(req.map(Body::from))
        .and_then(|res| {
            let status = res.status().clone();
            res.into_body().concat2().join(Ok(status))
        })
        .and_then(|(body, status)| {
            Ok(Response::builder()
                .status(status)
                .body(Bytes::from(body))
                .unwrap())
        })
        .map_err(RuntimeError::Http)
}

#[test]
fn example() {
    use tokio::runtime::current_thread::Runtime;
    let client = ClientWrapper::new();
    let policy = RetryPolicy::new(3);
    let mut svc = Retry::new(policy, client);

    let request = Request::builder()
        .uri("http://httpbin.org/json")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let mut runtime = Runtime::new().unwrap();
    let f = svc.call(request);
    let res = runtime.block_on(f);
    println!("{:?}", res);
}