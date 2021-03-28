use crate::Error;
use http::{uri::Scheme, Request, Response, Uri};
use hyper::{client::HttpConnector, Body};
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct Client<C = HttpConnector> {
    pub(crate) base: Uri,
    pub(crate) client: hyper::Client<C>,
}

impl<C> Client<C>
where
    C: hyper::client::connect::Connect + Sync + Send + Clone + 'static,
{
    pub fn with(base: Uri, connector: C) -> Self {
        let client = hyper::Client::builder().build(connector);
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
            .build();

        match uri {
            Ok(u) => {
                parts.uri = u;
                Ok(Request::from_parts(parts, body))
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub(crate) async fn call(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, body);
        let response = self.client.request(req).await?;
        Ok(response)
    }
}

#[cfg(test)]
mod endpoint_tests {
    use crate::{
        client::Client,
        incoming,
        requests::{
            EventCompletionRequest, EventErrorRequest, IntoRequest, IntoResponse, NextEventRequest, NextEventResponse,
        },
        simulated,
        types::Diagnostic,
        Error, Runtime,
    };
    use http::{uri::PathAndQuery, HeaderValue, Method, Request, Response, StatusCode, Uri};
    use hyper::{server::conn::Http, service::service_fn, Body};
    use serde_json::json;
    use simulated::DuplexStreamWrapper;
    use std::{convert::TryFrom, env};
    use tokio::{
        io::{self, AsyncRead, AsyncWrite},
        select,
        sync::{self, oneshot},
    };
    use tokio_stream::StreamExt;

    #[cfg(test)]
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
        rsp.into_rsp()
    }

    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
    async fn handle_incoming(req: Request<Body>) -> Result<Response<Body>, Error> {
        let path: Vec<&str> = req
            .uri()
            .path_and_query()
            .expect("PathAndQuery not found")
            .as_str()
            .split('/')
            .collect::<Vec<&str>>();
        match path[1..] {
            ["2018-06-01", "runtime", "invocation", "next"] => next_event(&req).await,
            ["2018-06-01", "runtime", "invocation", id, "response"] => complete_event(&req, id).await,
            ["2018-06-01", "runtime", "invocation", id, "error"] => event_err(&req, id).await,
            ["2018-06-01", "runtime", "init", "error"] => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    #[cfg(test)]
    async fn handle<I>(io: I, rx: oneshot::Receiver<()>) -> Result<(), hyper::Error>
    where
        I: AsyncRead + AsyncWrite + Unpin + 'static,
    {
        let conn = Http::new().serve_connection(io, service_fn(handle_incoming));
        select! {
            _ = rx => {
                Ok(())
            }
            res = conn => {
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        Err(e)
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_next_event() -> Result<(), Error> {
        let base = Uri::from_static("http://localhost:9001");
        let (client, server) = io::duplex(64);

        let (tx, rx) = sync::oneshot::channel();
        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

        let req = NextEventRequest.into_req()?;
        let rsp = client.call(req).await.expect("Unable to send request");

        assert_eq!(rsp.status(), StatusCode::OK);
        let header = "lambda-runtime-deadline-ms";
        assert_eq!(rsp.headers()[header], &HeaderValue::try_from("1542409706888")?);

        // shutdown server...
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn test_ok_response() -> Result<(), Error> {
        let (client, server) = io::duplex(64);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

        let req = EventCompletionRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            body: "done",
        };
        let req = req.into_req()?;

        let rsp = client.call(req).await?;
        assert_eq!(rsp.status(), StatusCode::ACCEPTED);

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn test_error_response() -> Result<(), Error> {
        let (client, server) = io::duplex(200);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });

        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;
        let client = Client::with(base, conn);

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

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }

    #[tokio::test]
    async fn successful_end_to_end_run() -> Result<(), Error> {
        let (client, server) = io::duplex(64);
        let (tx, rx) = sync::oneshot::channel();
        let base = Uri::from_static("http://localhost:9001");

        let server = tokio::spawn(async {
            handle(server, rx).await.expect("Unable to handle request");
        });
        let conn = simulated::Connector::with(base.clone(), DuplexStreamWrapper::new(client))?;

        let runtime = Runtime::builder()
            .with_endpoint(base)
            .with_connector(conn)
            .build()
            .expect("Unable to build runtime");

        async fn func(event: serde_json::Value, _: crate::Context) -> Result<serde_json::Value, Error> {
            Ok(event)
        }
        let f = crate::handler_fn(func);

        // set env vars needed to init Config if they are not already set in the environment
        if env::var("AWS_LAMBDA_RUNTIME_API").is_err() {
            env::set_var("AWS_LAMBDA_RUNTIME_API", "http://localhost:9001");
        }
        if env::var("AWS_LAMBDA_FUNCTION_NAME").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test_fn");
        }
        if env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "128");
        }
        if env::var("AWS_LAMBDA_FUNCTION_VERSION").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "1");
        }
        if env::var("AWS_LAMBDA_LOG_STREAM_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_STREAM_NAME", "test_stream");
        }
        if env::var("AWS_LAMBDA_LOG_GROUP_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_GROUP_NAME", "test_log");
        }
        let config = crate::Config::from_env().expect("Failed to read env vars");

        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        runtime.run(incoming, f, &config).await?;

        // shutdown server
        tx.send(()).expect("Receiver has been dropped");
        match server.await {
            Ok(_) => Ok(()),
            Err(e) if e.is_panic() => Err::<(), Error>(e.into()),
            Err(_) => unreachable!("This branch shouldn't be reachable"),
        }
    }
}
