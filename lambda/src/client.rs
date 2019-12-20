use crate::{
    requests::{
        EventCompletionRequest, EventErrorRequest, IntoRequest, IntoResponse, NextEventRequest,
        NextEventResponse,
    },
    support::http,
    types::Diagnostic,
    Err,
};
use bytes::buf::BufExt as _;
use http::{uri::PathAndQuery, HeaderValue, Method, Request, Response, StatusCode, Uri};
use hyper::Body;
use std::convert::{TryFrom, TryInto};
use tower_service::Service;

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

#[tokio::test]
async fn next_event() -> Result<(), Err> {
    let path = "/runtime/invocation/next";
    let server = http(move |req| {
        async move {
            assert_eq!(req.method(), Method::GET);
            assert_eq!(
                req.uri().path_and_query().unwrap(),
                &http::uri::PathAndQuery::from_static(path)
            );

            let rsp = NextEventResponse {
                request_id: "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
                deadline: 1542409706888,
                arn: "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
                trace_id: "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419",
                body: vec![],
            };
            let rsp = rsp.into_rsp().unwrap();
            rsp
        }
    });

    let url = format!("http://{}/", server.addr());
    let mut client = Client::with(url, hyper::Client::new())?;
    let rsp = client.call(NextEventRequest.into_req()?).await?;
    assert_eq!(rsp.status(), StatusCode::OK);
    assert_eq!(
        rsp.headers()["lambda-runtime-deadline-ms"],
        &HeaderValue::try_from("1542409706888").unwrap()
    );

    Ok(())
}

#[tokio::test]
async fn ok_response() -> Result<(), Err> {
    let id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
    let server = http(move |req| {
        let path = format!("/runtime/invocation/{}/response", id);

        async move {
            assert_eq!(req.method(), Method::POST);
            assert_eq!(
                req.uri().path_and_query().unwrap(),
                &path.parse::<PathAndQuery>().unwrap()
            );

            Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::empty())
                .unwrap()
        }
    });

    let req = EventCompletionRequest {
        request_id: id,
        body: "done",
    };

    let url = format!("http://{}/", server.addr());
    let mut client = Client::with(url, hyper::Client::new())?;
    let rsp = client.call(req.into_req()?).await?;
    assert_eq!(rsp.status(), StatusCode::ACCEPTED);

    Ok(())
}

#[tokio::test]
async fn error_response() -> Result<(), Err> {
    let id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
    let server = http(move |req| {
        let path = format!("/runtime/invocation/{}/error", id);

        async move {
            let (parts, body) = req.into_parts();
            let expected = Diagnostic {
                error_type: "InvalidEventDataError".to_string(),
                error_message: "Error parsing event data".to_string(),
            };

            let body = hyper::body::aggregate(body).await.unwrap();
            let actual = serde_json::from_reader(body.reader()).unwrap();
            assert_eq!(expected, actual);

            assert_eq!(parts.method, Method::POST);
            assert_eq!(
                parts.uri.path_and_query().unwrap(),
                &path.parse::<PathAndQuery>().unwrap()
            );
            let expected = "unhandled";
            assert_eq!(
                parts.headers["lambda-runtime-function-error-type"],
                HeaderValue::try_from(expected).unwrap()
            );

            Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::empty())
                .unwrap()
        }
    });

    let req = EventErrorRequest {
        request_id: id,
        diagnostic: Diagnostic {
            error_type: "InvalidEventDataError".to_string(),
            error_message: "Error parsing event data".to_string(),
        },
    };

    let url = format!("http://{}/", server.addr());
    let mut client = Client::with(url, hyper::Client::new())?;
    let rsp = client.call(req.into_req()?).await?;
    assert_eq!(rsp.status(), StatusCode::ACCEPTED);

    Ok(())
}
