use anyhow::Error;
use http::{Method, Request, Uri};
use serde::Serialize;
use std::str::FromStr;

pub(crate) trait IntoRequest {
    fn into_req(self) -> Result<http::Request<Vec<u8>>, Error>;
}

//   /runtime/invocation/next
#[derive(Debug, PartialEq)]
pub(crate) struct NextEventRequest;

impl IntoRequest for NextEventRequest {
    fn into_req(self) -> Result<http::Request<Vec<u8>>, Error> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_static("/runtime/invocation/next"))
            .body(Vec::new())?;
        Ok(req)
    }
}

#[test]
fn test_next_event_request() {
    let req = NextEventRequest;
    let req = req.into_req().unwrap();
    assert_eq!(req.method(), Method::GET);
    assert_eq!(req.uri(), &Uri::from_static("/runtime/invocation/next"));
}

// /runtime/invocation/{AwsRequestId}/response
pub(crate) struct EventCompletionRequest<'a, T> {
    pub(crate) request_id: &'a str,
    pub(crate) body: T,
}

impl<'a, T> IntoRequest for EventCompletionRequest<'a, T>
where
    T: for<'serialize> Serialize,
{
    fn into_req(self) -> Result<http::Request<Vec<u8>>, Error> {
        let uri = format!("/runtime/invocation/{}/response", self.request_id);
        let uri = Uri::from_str(&uri)?;
        let body = serde_json::to_vec(&self.body)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(body)?;
        Ok(req)
    }
}

#[test]
fn test_event_completion_request() {
    let req = EventCompletionRequest {
        request_id: "id",
        body: "hello, world!",
    };
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/runtime/invocation/id/response");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
}

// /runtime/invocation/{AwsRequestId}/error
pub(crate) struct EventErrorRequest<'a, T> {
    pub(crate) request_id: &'a str,
    pub(crate) body: T,
}

impl<'a, T> IntoRequest for EventErrorRequest<'a, T>
where
    T: for<'serialize> Serialize,
{
    fn into_req(self) -> Result<http::Request<Vec<u8>>, Error> {
        let uri = format!("/runtime/invocation/{}/error", self.request_id);
        let uri = Uri::from_str(&uri)?;
        let body = serde_json::to_vec(&self.body)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("lambda-runtime-function-error-type", "unhandled")
            .body(body)?;
        Ok(req)
    }
}

#[test]
fn test_event_error_request() {
    let req = EventErrorRequest {
        request_id: "id",
        body: "hello, world!",
    };
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/runtime/invocation/id/error");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
}

// /runtime/init/error
struct InitErrorRequest;

impl IntoRequest for InitErrorRequest {
    fn into_req(self) -> Result<http::Request<Vec<u8>>, Error> {
        let uri = format!("/runtime/init/error");
        let uri = Uri::from_str(&uri)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("lambda-runtime-function-error-type", "unhandled")
            .body(Vec::new())?;
        Ok(req)
    }
}

#[test]
fn test_init_error_request() {
    let req = InitErrorRequest;
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/runtime/init/error");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
}
