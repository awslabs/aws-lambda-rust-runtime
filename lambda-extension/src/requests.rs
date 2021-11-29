use crate::Error;
use http::{Method, Request};
use hyper::Body;
use lambda_runtime_client::build_request;
use serde::Serialize;

const EXTENSION_NAME_HEADER: &str = "Lambda-Extension-Name";
pub(crate) const EXTENSION_ID_HEADER: &str = "Lambda-Extension-Identifier";
const EXTENSION_ERROR_TYPE_HEADER: &str = "Lambda-Extension-Function-Error-Type";

pub(crate) fn next_event_request(extension_id: &str) -> Result<Request<Body>, Error> {
    let req = build_request()
        .method(Method::GET)
        .header(EXTENSION_ID_HEADER, extension_id)
        .uri("/2020-01-01/extension/event/next")
        .body(Body::empty())?;
    Ok(req)
}

pub(crate) fn register_request(extension_name: &str) -> Result<Request<Body>, Error> {
    let events = serde_json::json!({
        "events": ["INVOKE", "SHUTDOWN"]
    });

    let req = build_request()
        .method(Method::POST)
        .uri("/2020-01-01/extension/register")
        .header(EXTENSION_NAME_HEADER, extension_name)
        .body(Body::from(serde_json::to_string(&events)?))?;

    Ok(req)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRequest<'a> {
    error_message: &'a str,
    error_type: &'a str,
    stack_trace: Vec<&'a str>,
}

pub fn init_error<'a>(
    extension_id: &str,
    error_type: &str,
    request: Option<ErrorRequest<'a>>,
) -> Result<Request<Body>, Error> {
    error_request("init", extension_id, error_type, request)
}

pub fn exit_error<'a>(
    extension_id: &str,
    error_type: &str,
    request: Option<ErrorRequest<'a>>,
) -> Result<Request<Body>, Error> {
    error_request("exit", extension_id, error_type, request)
}

fn error_request<'a>(
    error_type: &str,
    extension_id: &str,
    error_str: &str,
    request: Option<ErrorRequest<'a>>,
) -> Result<Request<Body>, Error> {
    let uri = format!("/2020-01-01/extension/{}/error", error_type);

    let body = match request {
        None => Body::empty(),
        Some(err) => Body::from(serde_json::to_string(&err)?),
    };

    let req = build_request()
        .method(Method::POST)
        .uri(uri)
        .header(EXTENSION_ID_HEADER, extension_id)
        .header(EXTENSION_ERROR_TYPE_HEADER, error_str)
        .body(body)?;
    Ok(req)
}
