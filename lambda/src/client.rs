use crate::{
    requests::{IntoResponse, NextEventResponse},
    Err,
};
use futures::future;
use http::{
    uri::{PathAndQuery, Scheme},
    HeaderValue, Method, Request, Response, StatusCode, Uri,
};
use hyper::{client::HttpConnector, Body};
use serde_json::json;
use std::{
    convert::TryFrom,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_service::Service;

type Fut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

#[derive(Debug)]
pub(crate) struct Client<C = HttpConnector> {
    base: Uri,
    client: hyper::Client<C>,
}

impl<C> Client<C>
where
    C: hyper::client::connect::Connect + Sync + Send + Clone + 'static,
{
    pub fn with(base: Uri, client: hyper::Client<C>) -> Self {
        Self { base, client }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Err> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self.base.scheme().unwrap_or(&Scheme::HTTP);
            let authority = self.base.authority().expect("Authority not found");
            (scheme, authority)
        };
        let path = parts.uri.path_and_query().expect("PathAndQuery not found");

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build()
            .expect("Unable to build URI");

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }

    pub(crate) async fn call(&self, req: Request<Body>) -> Result<Response<Body>, Err> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.request(req).await?;
        Ok(response)
    }
}

pub struct EndpointSvc;

impl Service<Request<Body>> for EndpointSvc {
    type Response = Response<Body>;
    type Error = crate::Err;
    type Future = Fut<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let fut = async move {
            let path: Vec<&str> = req
                .uri()
                .path_and_query()
                .unwrap()
                .as_str()
                .split("/")
                .collect::<Vec<&str>>();
            match &path[1..] {
                ["2018-06-01", "runtime", "invocation", "next"] => next_event(&req).await,
                ["2018-06-01", "runtime", "invocation", id, "response"] => complete_event(&req, id).await,
                ["2018-06-01", "runtime", "invocation", id, "error"] => event_err(&req, id).await,
                ["2018-06-01", "runtime", "init", "error"] => unimplemented!(),
                _ => unimplemented!(),
            }
        };
        Box::pin(fut)
    }
}

async fn next_event(req: &Request<Body>) -> Result<Response<Body>, Err> {
    let path = "/2018-06-01/runtime/invocation/next";
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri().path_and_query().unwrap(), &PathAndQuery::from_static(path));
    let body = json!({"message": "hello"});

    let rsp = NextEventResponse {
        request_id: "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
        deadline: 1542409706888,
        arn: "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
        trace_id: "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419",
        body: serde_json::to_vec(&body)?,
    };
    rsp.into_rsp()
}

async fn complete_event(req: &Request<Body>, id: &str) -> Result<Response<Body>, Err> {
    assert_eq!(Method::POST, req.method());
    let rsp = Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .expect("Unable to construct response");

    let expected = format!("/2018-06-01/runtime/invocation/{}/response", id);
    assert_eq!(expected, req.uri().path());

    Ok(rsp)
}

async fn event_err(req: &Request<Body>, id: &str) -> Result<Response<Body>, Err> {
    let expected = format!("/2018-06-01/runtime/invocation/{}/error", id);
    assert_eq!(expected, req.uri().path());

    assert_eq!(req.method(), Method::POST);
    let header = "lambda-runtime-function-error-type";
    let expected = "unhandled";
    assert_eq!(req.headers()[header], HeaderValue::try_from(expected)?);

    let rsp = Response::builder().status(StatusCode::ACCEPTED).body(Body::empty())?;
    Ok(rsp)
}

pub struct MakeSvc;

impl<T> Service<T> for MakeSvc {
    type Response = EndpointSvc;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future {
        future::ok(EndpointSvc)
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::{Client, MakeSvc};
    use crate::{
        handler_fn,
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        run_simulated,
        types::Diagnostic,
        Err, INVOCATION_CTX,
    };
    use http::{HeaderValue, StatusCode};
    use std::{
        convert::{TryFrom, TryInto},
        net::{SocketAddr, TcpListener},
    };
    use tokio::select;

    /// `race` selects over two tasks.
    ///
    /// The first task to complete is joined and checked for errors.
    /// In this test suite, we don't expect that the "server" task
    /// will ever complete because it is continuously listening for
    /// incoming events.
    macro_rules! race {
        ($left:ident, $right:ident) => {
            select! {
                $left = $left => { $left?? },
                $right = $right => { $right?? }
            };
        };
    }

    fn setup() -> Result<(TcpListener, SocketAddr), Err> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        Ok((listener, addr))
    }

    #[tokio::test]
    async fn next_event() -> Result<(), Err> {
        let (listener, addr) = setup()?;
        let url = format!("http://{}/", addr);

        let server = tokio::spawn(async move {
            let svc = hyper::Server::from_tcp(listener)?.serve(MakeSvc);
            svc.await
        });

        let client = tokio::spawn(async {
            let url = url.try_into().expect("Unable to convert to URL");
            let client = Client::with(url, hyper::Client::new());
            let req = NextEventRequest.into_req()?;
            let rsp = client.call(req).await?;

            assert_eq!(rsp.status(), StatusCode::OK);
            let header = "lambda-runtime-deadline-ms";
            assert_eq!(rsp.headers()[header], &HeaderValue::try_from("1542409706888")?);
            Ok::<(), Err>(())
        });
        race!(client, server);
        Ok(())
    }

    #[tokio::test]
    async fn ok_response() -> Result<(), Err> {
        let (listener, addr) = setup()?;
        let url = format!("http://{}/", addr);

        let server = tokio::spawn(async move {
            let svc = hyper::Server::from_tcp(listener)?.serve(MakeSvc);
            svc.await
        });

        let client = tokio::spawn(async {
            let url = url.try_into().expect("Unable to convert to URL");
            let client = Client::with(url, hyper::Client::new());
            let req = EventCompletionRequest {
                request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
                body: "done",
            };
            let req = req.into_req()?;
            let rsp = client.call(req).await?;
            assert_eq!(rsp.status(), StatusCode::ACCEPTED);
            Ok::<(), Err>(())
        });
        race!(client, server);
        Ok(())
    }

    #[tokio::test]
    async fn error_response() -> Result<(), Err> {
        let (listener, addr) = setup()?;
        let url = format!("http://{}/", addr);

        let server = tokio::spawn(async move {
            let svc = hyper::Server::from_tcp(listener)?.serve(MakeSvc);
            svc.await
        });

        let client = tokio::spawn(async {
            let url = url.try_into().expect("Unable to convert to URL");
            let client = Client::with(url, hyper::Client::new());
            let req = EventErrorRequest {
                request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
                diagnostic: Diagnostic {
                    error_type: "InvalidEventDataError".to_string(),
                    error_message: "Error parsing event data".to_string(),
                },
            };
            let req = req.into_req()?;
            let rsp = client.call(req).await?;
            assert_eq!(rsp.status(), StatusCode::ACCEPTED);
            Ok::<(), Err>(())
        });
        race!(client, server);
        Ok(())
    }

    #[tokio::test]
    async fn run_end_to_end() -> Result<(), Err> {
        use serde_json::Value;
        let (listener, addr) = setup()?;
        let url = format!("http://{}/", addr);

        let server = tokio::spawn(async move {
            let svc = hyper::Server::from_tcp(listener)?.serve(MakeSvc);
            svc.await
        });

        async fn handler(s: Value) -> Result<Value, Err> {
            INVOCATION_CTX.with(|ctx| {});
            Ok(s)
        }
        let handler = handler_fn(handler);
        let client = tokio::spawn(async move {
            run_simulated(handler, &url).await?;
            Ok::<(), Err>(())
        });
        race!(client, server);
        Ok(())
    }
}
