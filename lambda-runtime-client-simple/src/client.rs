use bytes::Bytes;
use futures::{Async, Future};
use hyper::{
    client::{
        connect::{Connect, Destination},
        HttpConnector,
    },
    Body, Request, Response, Uri,
};
use std::time::Duration;
use tower::BoxService as ServiceObj;
use tower_add_origin::{AddOrigin, Builder};
use tower_hyper::{client::Client, retry::RetryPolicy};
use tower_retry::Retry;
use tower_service::Service;
use tower_timeout::Timeout;

type Err = Box<dyn std::error::Error + Send + Sync>;

pub(crate) struct LambdaSvc {
    inner: ServiceObj<Request<Body>, Response<Body>, Err>,
}

impl LambdaSvc {
    fn new<C>(origin: Uri, connector: C) -> Self
    where
        C: Connect + 'static,
        C::Transport: 'static,
        C::Future: 'static,
    {
        let svc = Client::new(hyper::Client::builder().build::<_, hyper::Body>(connector));
        let svc = Timeout::new(svc, Duration::from_millis(300));
        let svc = Builder::new().uri(origin).build(svc).unwrap();
        let svc = ServiceObj::new(svc);
        LambdaSvc { inner: svc }
    }
}

impl Service<Request<Body>> for LambdaSvc {
    type Response = Response<Body>;
    type Error = Err;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send>;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, origin: Request<Body>) -> Self::Future {
        let request = Request::builder()
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let f = self.inner.call(request).map_err(|e| panic!("{:?}", e));
        Box::new(f)
    }
}

#[test]
fn get_next_event() {
    use crate::mock::MockConnector;
    use tokio::runtime::current_thread::Runtime;

    let mut rt = Runtime::new().expect("new rt");
    let origin = Uri::from_static("http://localhost:9000");
    let mut conn = MockConnector::new();
    conn.mock("http://localhost:9000");

    let req = Request::get("/2018-06-01/runtime/invocation/next")
        .body(Body::empty())
        .unwrap();

    let mut svc = LambdaSvc::new(origin, conn);
    let res = svc.call(req);
    let res = rt.block_on(res);
}
