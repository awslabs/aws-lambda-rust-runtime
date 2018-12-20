//! Response types

// Std
use std::ops::Not;

use http::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Response,
};
use serde::{
    ser::{Error as SerError, SerializeMap},
    Serializer,
};
use serde_derive::Serialize;

use crate::body::Body;

/// Representation of API Gateway response
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GatewayResponse {
    pub status_code: u16,
    #[serde(skip_serializing_if = "HeaderMap::is_empty", serialize_with = "serialize_headers")]
    pub headers: HeaderMap<HeaderValue>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        serialize_with = "serialize_multi_value_headers"
    )]
    pub multi_value_headers: HeaderMap<HeaderValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(skip_serializing_if = "Not::not")]
    pub is_base64_encoded: bool,
}

#[cfg(test)]
impl Default for GatewayResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Default::default(),
            is_base64_encoded: Default::default(),
        }
    }
}

/// Serialize a http::HeaderMap into a serde str => str map
fn serialize_multi_value_headers<S>(headers: &HeaderMap<HeaderValue>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;
    for key in headers.keys() {
        let mut map_values = Vec::new();
        for value in headers.get_all(key) {
            map_values.push(value.to_str().map_err(S::Error::custom)?)
        }
        map.serialize_entry(key.as_str(), &map_values)?;
    }
    map.end()
}

/// Serialize a http::HeaderMap into a serde str => Vec<str> map
fn serialize_headers<S>(headers: &HeaderMap<HeaderValue>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;
    for key in headers.keys() {
        let map_value = headers[key].to_str().map_err(S::Error::custom)?;
        map.serialize_entry(key.as_str(), map_value)?;
    }
    map.end()
}

impl<T> From<Response<T>> for GatewayResponse
where
    T: Into<Body>,
{
    fn from(value: Response<T>) -> Self {
        let (parts, bod) = value.into_parts();
        let (is_base64_encoded, body) = match bod.into() {
            Body::Empty => (false, None),
            b @ Body::Text(_) => (false, Some(b)),
            b @ Body::Binary(_) => (true, Some(b)),
        };
        GatewayResponse {
            status_code: parts.status.as_u16(),
            body,
            headers: parts.headers.clone(),
            multi_value_headers: parts.headers,
            is_base64_encoded,
        }
    }
}

/// A conversion of self into a `Response`
///
/// Implementations for `Response<B> where B: Into<Body>`,
/// `B where B: Into<Body>` and `serde_json::Value` are provided
/// by default
///
/// # example
///
/// ```rust
/// use lambda_http::{Body, IntoResponse, Response};
///
/// assert_eq!(
///   "hello".into_response().body(),
///   Response::new(Body::from("hello")).body()
/// );
/// ```
pub trait IntoResponse {
    /// Return a translation of `self` into a `Response<Body>`
    fn into_response(self) -> Response<Body>;
}

impl<B> IntoResponse for Response<B>
where
    B: Into<Body>,
{
    fn into_response(self) -> Response<Body> {
        let (parts, body) = self.into_parts();
        Response::from_parts(parts, body.into())
    }
}

impl<B> IntoResponse for B
where
    B: Into<Body>,
{
    fn into_response(self) -> Response<Body> {
        Response::new(self.into())
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_string(&self)
                    .expect("unable to serialize serde_json::Value")
                    .into(),
            )
            .expect("unable to build http::Response")
    }
}

#[cfg(test)]
mod tests {
    use super::{Body, GatewayResponse, IntoResponse};
    use http::Response;
    use serde_json::{self, json};

    #[test]
    fn json_into_response() {
        let response = json!({ "hello": "lambda"}).into_response();
        match response.body() {
            Body::Text(json) => assert_eq!(json, r#"{"hello":"lambda"}"#),
            _ => panic!("invalid body"),
        }
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h.to_str().expect("invalid header")),
            Some("application/json")
        )
    }

    #[test]
    fn text_into_response() {
        let response = "text".into_response();
        match response.body() {
            Body::Text(text) => assert_eq!(text, "text"),
            _ => panic!("invalid body"),
        }
    }

    #[test]
    fn default_response() {
        assert_eq!(GatewayResponse::default().status_code, 200)
    }

    #[test]
    fn serialize_default() {
        assert_eq!(
            serde_json::to_string(&GatewayResponse::default()).expect("failed to serialize response"),
            r#"{"statusCode":200}"#
        );
    }

    #[test]
    fn serialize_body() {
        let mut resp = GatewayResponse::default();
        resp.body = Some("foo".into());
        assert_eq!(
            serde_json::to_string(&resp).expect("failed to serialize response"),
            r#"{"statusCode":200,"body":"foo"}"#
        );
    }

    #[test]
    fn serialize_multi_value_headers() {
        let res: GatewayResponse = Response::builder()
            .header("multi", "a")
            .header("multi", "b")
            .body(Body::from(()))
            .expect("failed to create request")
            .into();
        let json = serde_json::to_string(&res).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"multi":"a"},"multiValueHeaders":{"multi":["a","b"]}}"#
        )
    }
}
