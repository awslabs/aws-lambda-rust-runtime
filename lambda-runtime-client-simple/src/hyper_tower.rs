use bytes::Bytes;
use futures::{future, stream::Stream, Async, Future, Poll};
use http::{Request, Response};
use hyper::{client::HttpConnector, Body, Client};
use tower_retry::{Policy, Retry};
use tower_service::Service;

use crate::RuntimeError;

#[derive(Clone)]
pub(crate) struct ClientWrapper {
    inner: Client<HttpConnector>,
}

impl ClientWrapper {
    fn new() -> Self {
        ClientWrapper { inner: Client::new() }
    }
}

pub type HttpFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

impl Service<Request<Bytes>> for ClientWrapper {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = HttpFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Request<Bytes>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, Body::from(body));
        let f = self.inner.request(req);

        Box::new(f)
    }
}

#[derive(Clone)]
struct RetryPolicy {
    attempts: u8,
}

impl RetryPolicy {
    fn new(attempts: u8) -> Self {
        RetryPolicy { attempts }
    }
}

impl Policy<Request<Bytes>, Response<Body>, hyper::Error> for RetryPolicy {
    type Future = future::FutureResult<Self, ()>;

    fn retry(&self, _: &Request<Bytes>, result: Result<&Response<Body>, &hyper::Error>) -> Option<Self::Future> {
        if self.attempts == 0 {
            // We ran out of retries, hence us returning none.
            return None;
        }

        match result {
            Ok(res) => {
                if res.status().is_server_error() {
                    let policy = RetryPolicy::new(self.attempts - 1);
                    Some(future::ok(policy))
                } else {
                    // 2xx-4xx shouldn't be retried.
                    None
                }
            }
            Err(_) => Some(future::ok(RetryPolicy {
                attempts: self.attempts - 1,
            })),
        }
    }

    fn clone_request(&self, req: &Request<Bytes>) -> Option<Request<Bytes>> {
        // there is no .parts(&self) method on request.
        let body = req.body().clone();
        let mut clone = http::Request::new(body);
        *clone.uri_mut() = req.uri().clone();
        *clone.headers_mut() = req.headers().clone();
        *clone.method_mut() = req.method().clone();
        *clone.method_mut() = req.method().clone();
        *clone.version_mut() = req.version().clone();
        Some(clone)
    }
}

pub(crate) fn hyper(req: Request<Bytes>) -> impl Future<Item = Response<Bytes>, Error = RuntimeError> {
    let svc = ClientWrapper::new();
    let policy = RetryPolicy::new(3);
    let mut svc = Retry::new(policy, svc);

    svc.call(req)
        .map_err(|e| e)
        .and_then(|res| {
            let status = res.status().clone();
            res.into_body().concat2().join(Ok(status))
        })
        .and_then(|(body, status)| Ok(Response::builder().status(status).body(Bytes::from(body)).unwrap()))
        .map_err(RuntimeError::Http)
}

#[test]
fn example() {
    use http::Method;

    use tokio::runtime::current_thread::Runtime;
    let client = ClientWrapper::new();
    let policy = RetryPolicy::new(3);
    let mut svc = Retry::new(policy, client);

    let request = Request::builder()
        .uri("http://httpbin.org/json")
        .method(Method::GET)
        .body(Bytes::new())
        .unwrap();

    let mut runtime = Runtime::new().unwrap();
    let f = svc.call(request);
    let res = runtime.block_on(f);
    println!("{:?}", res);
}
