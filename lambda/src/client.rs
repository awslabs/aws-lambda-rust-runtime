use crate::{
    requests::{IntoResponse, NextEventResponse},
    types::Diagnostic,
    Err,
};
use bytes::buf::ext::BufExt;
use futures::future;
use http::{uri::PathAndQuery, HeaderValue, Method, Request, Response, StatusCode, Uri};
use hyper::Body;
use std::{
    convert::{TryFrom, TryInto},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_service::Service;

type Fut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

#[derive(Debug, Clone)]
pub(crate) struct Client<S> {
    base: Uri,
    client: S,
}

impl<S> Client<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
    <S as Service<Request<Body>>>::Error: Into<Err> + Send + Sync + 'static + std::error::Error,
{
    pub fn with<T>(base: T, client: S) -> Result<Self, Err>
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: std::error::Error + Send + Sync + 'static,
    {
        let base = base.try_into()?;
        Ok(Self { base, client })
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Err> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self.base.scheme().expect("Scheme not found");
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

    pub(crate) async fn call(&mut self, req: Request<Body>) -> Result<Response<Body>, Err> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.call(req).await?;
        Ok(response)
    }
}

pub struct NextEventSvc;

impl Service<Request<Body>> for NextEventSvc {
    type Response = Response<Body>;
    type Error = crate::Err;
    type Future = Fut<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let fut = async move {
            let path = req.uri().path_and_query().unwrap().as_str();
            let rsp = if path.ends_with("next") {
                next_event(req).await
            } else if path.ends_with("response") {
                complete_event(req).await
            } else {
                event_err(req).await
            };
            rsp
        };
        Box::pin(fut)
    }
}

async fn next_event(req: Request<Body>) -> Result<Response<Body>, Err> {
    let path = "/2018-06-01/runtime/invocation/next";
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri().path_and_query().unwrap(), &PathAndQuery::from_static(path));

    let rsp = NextEventResponse {
        request_id: "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
        deadline: 1542409706888,
        arn: "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
        trace_id: "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419",
        body: vec![],
    };
    rsp.into_rsp()
}

async fn complete_event(req: Request<Body>) -> Result<Response<Body>, Err> {
    assert_eq!(req.method(), Method::POST);
    let rsp = Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .expect("Unable to construct response");
    Ok(rsp)
}

async fn event_err(req: Request<Body>) -> Result<Response<Body>, Err> {
    let (parts, body) = req.into_parts();
    let expected = Diagnostic {
        error_type: "InvalidEventDataError".to_string(),
        error_message: "Error parsing event data".to_string(),
    };

    let body = hyper::body::aggregate(body).await?;
    let actual = serde_json::from_reader(body.reader())?;
    assert_eq!(expected, actual);

    assert_eq!(parts.method, Method::POST);
    let header = "lambda-runtime-function-error-type";
    let expected = "unhandled";
    assert_eq!(parts.headers[header], HeaderValue::try_from(expected)?);

    let rsp = Response::builder().status(StatusCode::ACCEPTED).body(Body::empty())?;
    Ok(rsp)
}

pub struct MakeSvc;

impl<T> Service<T> for MakeSvc {
    type Response = NextEventSvc;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future {
        future::ok(NextEventSvc)
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::{Client, MakeSvc};
    use crate::{
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        types::Diagnostic,
        Err,
    };
    use futures::future;
    use http::{HeaderValue, StatusCode};
    use std::{
        convert::TryFrom,
        net::{SocketAddr, TcpListener},
    };

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
            let mut client = Client::with(url, hyper::Client::new())?;
            let req = NextEventRequest.into_req()?;
            let rsp = client.call(req).await?;

            assert_eq!(rsp.status(), StatusCode::OK);
            let header = "lambda-runtime-deadline-ms";
            assert_eq!(rsp.headers()[header], &HeaderValue::try_from("1542409706888")?);
            Ok::<(), Err>(())
        });
        future::try_select(client, server).await.unwrap();
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
            let mut client = Client::with(url, hyper::Client::new())?;
            let req = EventCompletionRequest {
                request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
                body: "done",
            };
            let req = req.into_req()?;
            let rsp = client.call(req).await?;
            assert_eq!(rsp.status(), StatusCode::ACCEPTED);
            Ok::<(), Err>(())
        });
        future::try_select(server, client).await.unwrap();
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
            let mut client = Client::with(url, hyper::Client::new())?;
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
        future::try_select(server, client).await.unwrap();
        Ok(())
    }
}
