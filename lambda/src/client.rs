use crate::requests::{IntoRequest, NextEventRequest};
use crate::{err_fmt, Error};
use async_stream::try_stream;
use bytes::Bytes;
use futures::prelude::*;
use http::{uri::PathAndQuery, Method, Request, Response, Uri};
use hyper::Body;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Client {
    base: Uri,
    client: hyper::Client<hyper::client::HttpConnector>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LambdaError {
    error_type: String,
    error_message: String,
}

#[derive(Debug, PartialEq)]
struct InitializationErrorRequest {
    path: PathAndQuery,
    error: LambdaError,
}

impl InitializationErrorRequest {
    fn new(error: LambdaError) -> Self {
        Self {
            path: PathAndQuery::from_static("/runtime/init/error"),
            error,
        }
    }
}

#[derive(Debug, PartialEq)]
struct InvocationOkRequest {
    path: PathAndQuery,
    body: Vec<u8>,
}

impl InvocationOkRequest {
    fn new(request_id: String, body: Vec<u8>) -> Self {
        let path = format!("/runtime/invocation/{}/response", request_id);
        let path =
            PathAndQuery::from_shared(Bytes::from(path)).expect("Unable to construct PathAndQuery");
        Self { path, body }
    }
}
#[derive(Debug, PartialEq)]
struct InvocationErrorRequest {
    path: PathAndQuery,
    error: LambdaError,
}

impl InvocationErrorRequest {
    fn new(request_id: String, error: LambdaError) -> Self {
        let path = format!("/runtime/invocation/{}/error", request_id);
        let path =
            PathAndQuery::from_shared(Bytes::from(path)).expect("Unable to construct PathAndQuery");
        Self { path, error }
    }
}

#[test]
fn round_trip_lambda_error() -> Result<(), anyhow::Error> {
    use serde_json::{from_value, json, to_value, Value};
    let expected = json!({
        "errorMessage" : "Error parsing event data.",
        "errorType" : "InvalidEventDataException"
    });

    let actual: LambdaError = from_value(expected.clone())?;
    let actual: Value = to_value(actual)?;
    assert_eq!(expected, actual);

    Ok(())
}

impl Client {
    pub fn new(base: Uri) -> Self {
        Self {
            base,
            client: hyper::Client::new(),
        }
    }

    fn set_origin(&self, req: Request<Vec<u8>>) -> Result<Request<Vec<u8>>, anyhow::Error> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self
                .base
                .scheme_part()
                .ok_or(err_fmt!("PathAndQuery not found"))?;
            let authority = self
                .base
                .authority_part()
                .ok_or(err_fmt!("Authority not found"))?;
            (scheme, authority)
        };
        let path = parts
            .uri
            .path_and_query()
            .ok_or(err_fmt!("PathAndQuery not found"))?;

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build()
            .expect("Unable to build URI");

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }

    pub async fn call(&mut self, req: Request<Vec<u8>>) -> Result<Response<Body>, anyhow::Error> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.request(req).await.map_err(Error::Hyper)?;
        Ok(response)
    }
}

pub fn events(client: Client) -> impl Stream<Item = Result<Response<Body>, anyhow::Error>> {
    try_stream! {
        let mut client = client;
        loop {
            let req = NextEventRequest;
            let req = req.into_req()?;
            let event = client.call(req).await?;
            yield event;
        }
    }
}
