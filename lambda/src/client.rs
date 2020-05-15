use crate::requests::{IntoResponse, NextEventResponse};
use anyhow::Error;
use http::{
    uri::{PathAndQuery, Scheme},
    HeaderValue, Method, Request, Response, StatusCode, Uri,
};
use hyper::{client::HttpConnector, server::conn::Http, service::service_fn, Body};
use serde_json::json;
use std::convert::TryFrom;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    select,
    sync::oneshot,
};
use tracing::{error, info, instrument};

#[instrument]
async fn hello(req: Request<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("hello")))
}

async fn handle_incoming(req: Request<Body>) -> Result<Response<Body>, Error> {
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
}

#[instrument(skip(io, rx))]
async fn handle<I>(io: I, rx: oneshot::Receiver<()>) -> Result<(), hyper::error::Error>
where
    I: AsyncRead + AsyncWrite + Unpin + 'static,
{
    let conn = Http::new().serve_connection(io, service_fn(handle_incoming));
    select! {
        _ = rx => {
            info!("Received cancelation signal");
            return Ok(())
        }
        res = conn => {
            match res {
                Ok(()) => return Ok(()),
                Err(e) => {
                    error!(message = "Got error serving connection", e = %e);
                    return Err(e);
                }
            }
        }
    }
}

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

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Error> {
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

    pub(crate) async fn call(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.request(req).await?;
        Ok(response)
    }
}

async fn next_event(req: &Request<Body>) -> Result<Response<Body>, Error> {
    let path = "/2018-06-01/runtime/invocation/next";
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri().path_and_query().unwrap(), &PathAndQuery::from_static(path));
    let body = json!({"message": "hello"});

    let rsp = NextEventResponse {
        request_id: "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
        deadline: 1_542_409_706_888,
        arn: "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
        trace_id: "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419",
        body: serde_json::to_vec(&body)?,
    };
    rsp.into_rsp().map_err(|e| e.into())
}

async fn complete_event(req: &Request<Body>, id: &str) -> Result<Response<Body>, Error> {
    assert_eq!(Method::POST, req.method());
    let rsp = Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .expect("Unable to construct response");

    let expected = format!("/2018-06-01/runtime/invocation/{}/response", id);
    assert_eq!(expected, req.uri().path());

    Ok(rsp)
}

async fn event_err(req: &Request<Body>, id: &str) -> Result<Response<Body>, Error> {
    let expected = format!("/2018-06-01/runtime/invocation/{}/error", id);
    assert_eq!(expected, req.uri().path());

    assert_eq!(req.method(), Method::POST);
    let header = "lambda-runtime-function-error-type";
    let expected = "unhandled";
    assert_eq!(req.headers()[header], HeaderValue::try_from(expected)?);

    let rsp = Response::builder().status(StatusCode::ACCEPTED).body(Body::empty())?;
    Ok(rsp)
}

fn set_origin<B>(base: Uri, req: Request<B>) -> Result<Request<B>, Error> {
    let (mut parts, body) = req.into_parts();
    let (scheme, authority) = {
        let scheme = base.scheme().unwrap_or(&Scheme::HTTP);
        let authority = base.authority().expect("Authority not found");
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

#[cfg(test)]
mod endpoint_tests {
    use super::{handle, set_origin};
    use crate::{
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        simulated::SimulatedConnector,
        types::Diagnostic,
    };
    use anyhow::Error;
    use http::{HeaderValue, StatusCode, Uri};
    use std::convert::TryFrom;
    use tokio::sync;

    #[tokio::test]
    async fn next_event() -> Result<(), Error> {
        let (client, server) = crate::simulated::chan();
        let base = Uri::from_static("http://localhost:9001");

        let (tx, rx) = sync::oneshot::channel();
        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = SimulatedConnector { inner: client };
        let client = hyper::Client::builder().build(conn);

        let req = NextEventRequest.into_req()?;
        let req = set_origin(base, req)?;
        let rsp = client.request(req).await.expect("Unable to send request");

        assert_eq!(rsp.status(), StatusCode::OK);
        let header = "lambda-runtime-deadline-ms";
        assert_eq!(rsp.headers()[header], &HeaderValue::try_from("1542409706888")?);

        // shutdown server...
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => return Err::<(), anyhow::Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn ok_response() -> Result<(), Error> {
        let (client, server) = crate::simulated::chan();
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = SimulatedConnector { inner: client };
        let client = hyper::Client::builder().build(conn);

        let req = EventCompletionRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            body: "done",
        };
        let req = req.into_req()?;
        let req = set_origin(base, req)?;

        let rsp = client.request(req).await?;
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => return Err::<(), anyhow::Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn error_response() -> Result<(), Error> {
        let (client, server) = crate::simulated::chan();
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = SimulatedConnector { inner: client };
        let client = hyper::Client::builder().build(conn);

        let req = EventErrorRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            diagnostic: Diagnostic {
                error_type: "InvalidEventDataError".to_string(),
                error_message: "Error parsing event data".to_string(),
            },
        };
        let req = req.into_req()?;
        let req = set_origin(base, req)?;
        let rsp = client.request(req).await?;
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => return Err::<(), anyhow::Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    // #[tokio::test]
    // async fn run_end_to_end() -> Result<(), Error> {
    //     use serde_json::Value;

    //     let (client, server) = crate::simulated::chan();

    //     let (tx, rx) = sync::oneshot::channel();
    //     let server = tokio::spawn(async move { handle(server, rx) });

    //     async fn handler(s: Value) -> Result<Value, Error> {
    //         INVOCATION_CTX.with(|_ctx| {});
    //         Ok(s)
    //     }
    //     let handler = handler_fn(handler);
    //     let client = tokio::spawn(async move {
    //         run_simulated(handler, &url).await?;
    //         Ok::<(), Error>(())
    //     });
    //     race!(client, server);
    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_stream_handler() -> Result<(), Error> {
    //     let (client, server) = crate::simulated::chan();
    //     let req = Request::builder()
    //         .method(Method::GET)
    //         .uri("http://httpbin.org")
    //         .body(Body::empty())
    //         .expect("Can't build request");

    //     let conn = SimulatedConnector { inner: client };
    //     let client = hyper::Client::builder().build(conn);

    //     let (tx, rx) = sync::oneshot::channel();
    //     let server = tokio::spawn(async {
    //         handle(server, rx).await.expect("Unable to handle request");
    //     });

    //     let rsp = client.request(req).await.expect("Unable to send request");
    //     assert_eq!(rsp.status(), http::StatusCode::OK);

    //     // shutdown server
    //     tx.send(()).expect("Receiver has been dropped");
    //     match server.await {
    //         Ok(_) => Ok(()),
    //         Err(e) if e.is_panic() => return Err::<(), anyhow::Error>(e.into()),
    //         Err(_) => unreachable!("This branch shouldn't be reachable"),
    //     }
    // }
}
