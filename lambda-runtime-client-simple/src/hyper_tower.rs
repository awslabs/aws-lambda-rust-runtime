use bytes::Bytes;
use futures::{future, stream::Stream, Async, Future, Poll};
use http::{Request, Response};
use hyper::{
    client::{HttpConnector, ResponseFuture},
    Body, Client,
};
use tower_retry::{Policy, Retry};
use tower_service::Service;

use crate::Error;

#[derive(Clone)]
pub(crate) struct ClientWrapper {
    inner: Client<HttpConnector>,
}

impl ClientWrapper {
    fn new() -> Self {
        ClientWrapper { inner: Client::new() }
    }
}

impl Service<Request<Bytes>> for ClientWrapper {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Request<Bytes>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, Body::from(body));
        self.inner.request(req)
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

impl<T> Policy<Request<T>, Response<Body>, hyper::Error> for RetryPolicy
where
    T: Into<Body> + Clone,
{
    type Future = future::FutureResult<Self, ()>;

    fn retry(&self, _: &Request<T>, result: Result<&Response<Body>, &hyper::Error>) -> Option<Self::Future> {
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

    fn clone_request(&self, req: &Request<T>) -> Option<Request<T>> {
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

pub(crate) fn hyper(req: Request<Bytes>) -> impl Future<Item = Response<Bytes>, Error = Error> {
    let svc = ClientWrapper::new();
    let policy = RetryPolicy::new(3);
    let mut svc = Retry::new(policy, svc);

    svc.call(req)
        .and_then(|res| {
            let status = res.status().clone();
            res.into_body().concat2().join(Ok(status))
        })
        .and_then(|(body, status)| Ok(Response::builder().status(status).body(Bytes::from(body)).unwrap()))
        .map_err(|e| e.into())
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
