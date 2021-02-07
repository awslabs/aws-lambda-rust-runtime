//! Response types

use crate::{body::Body, request::RequestOrigin};
use http::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, SET_COOKIE},
    Response,
};
use serde::{
    ser::{Error as SerError, SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

/// Representation of Lambda response
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum LambdaResponse {
    ApiGatewayV2(ApiGatewayV2Response),
    Alb(AlbResponse),
    ApiGateway(ApiGatewayResponse),
}

/// Representation of API Gateway v2 lambda response
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2Response {
    status_code: u16,
    #[serde(serialize_with = "serialize_headers")]
    headers: HeaderMap<HeaderValue>,
    #[serde(serialize_with = "serialize_headers_slice")]
    cookies: Vec<HeaderValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Body>,
    is_base64_encoded: bool,
}

/// Representation of ALB lambda response
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbResponse {
    status_code: u16,
    status_description: String,
    #[serde(serialize_with = "serialize_headers")]
    headers: HeaderMap<HeaderValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Body>,
    is_base64_encoded: bool,
}

/// Representation of API Gateway lambda response
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayResponse {
    status_code: u16,
    #[serde(serialize_with = "serialize_headers")]
    headers: HeaderMap<HeaderValue>,
    #[serde(serialize_with = "serialize_multi_value_headers")]
    multi_value_headers: HeaderMap<HeaderValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Body>,
    is_base64_encoded: bool,
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

/// Serialize a &[HeaderValue] into a Vec<str>
fn serialize_headers_slice<S>(headers: &[HeaderValue], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(headers.len()))?;
    for header in headers {
        seq.serialize_element(header.to_str().map_err(S::Error::custom)?)?;
    }
    seq.end()
}

/// tranformation from http type to internal type
impl LambdaResponse {
    pub(crate) fn from_response<T>(request_origin: &RequestOrigin, value: Response<T>) -> Self
    where
        T: Into<Body>,
    {
        let (parts, bod) = value.into_parts();
        let (is_base64_encoded, body) = match bod.into() {
            Body::Empty => (false, None),
            b @ Body::Text(_) => (false, Some(b)),
            b @ Body::Binary(_) => (true, Some(b)),
        };

        let mut headers = parts.headers;
        let status_code = parts.status.as_u16();

        match request_origin {
            RequestOrigin::ApiGatewayV2 => {
                // ApiGatewayV2 expects the set-cookies headers to be in the "cookies" attribute,
                // so remove them from the headers.
                let cookies: Vec<HeaderValue> = headers.get_all(SET_COOKIE).iter().cloned().collect();
                headers.remove(SET_COOKIE);

                LambdaResponse::ApiGatewayV2(ApiGatewayV2Response {
                    body,
                    status_code,
                    is_base64_encoded,
                    cookies,
                    headers,
                })
            }
            RequestOrigin::ApiGateway => LambdaResponse::ApiGateway(ApiGatewayResponse {
                body,
                status_code,
                is_base64_encoded,
                headers: headers.clone(),
                multi_value_headers: headers,
            }),
            RequestOrigin::Alb => LambdaResponse::Alb(AlbResponse {
                body,
                status_code,
                is_base64_encoded,
                headers,
                status_description: format!(
                    "{} {}",
                    status_code,
                    parts.status.canonical_reason().unwrap_or_default()
                ),
            }),
        }
    }
}

/// A conversion of self into a `Response<Body>` for various types.
///
/// Implementations for `Response<B> where B: Into<Body>`,
/// `B where B: Into<Body>` and `serde_json::Value` are provided
/// by default.
///
/// # Example
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
    use super::{
        AlbResponse, ApiGatewayResponse, ApiGatewayV2Response, Body, IntoResponse, LambdaResponse, RequestOrigin,
    };
    use http::{header::CONTENT_TYPE, Response};
    use serde_json::{self, json};

    fn api_gateway_response() -> ApiGatewayResponse {
        ApiGatewayResponse {
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Default::default(),
            is_base64_encoded: Default::default(),
        }
    }

    fn alb_response() -> AlbResponse {
        AlbResponse {
            status_code: 200,
            status_description: "200 OK".to_string(),
            headers: Default::default(),
            body: Default::default(),
            is_base64_encoded: Default::default(),
        }
    }

    fn api_gateway_v2_response() -> ApiGatewayV2Response {
        ApiGatewayV2Response {
            status_code: 200,
            headers: Default::default(),
            body: Default::default(),
            cookies: Default::default(),
            is_base64_encoded: Default::default(),
        }
    }

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
    fn serialize_body_for_api_gateway() {
        let mut resp = api_gateway_response();
        resp.body = Some("foo".into());
        assert_eq!(
            serde_json::to_string(&resp).expect("failed to serialize response"),
            r#"{"statusCode":200,"headers":{},"multiValueHeaders":{},"body":"foo","isBase64Encoded":false}"#
        );
    }

    #[test]
    fn serialize_body_for_alb() {
        let mut resp = alb_response();
        resp.body = Some("foo".into());
        assert_eq!(
            serde_json::to_string(&resp).expect("failed to serialize response"),
            r#"{"statusCode":200,"statusDescription":"200 OK","headers":{},"body":"foo","isBase64Encoded":false}"#
        );
    }

    #[test]
    fn serialize_body_for_api_gateway_v2() {
        let mut resp = api_gateway_v2_response();
        resp.body = Some("foo".into());
        assert_eq!(
            serde_json::to_string(&resp).expect("failed to serialize response"),
            r#"{"statusCode":200,"headers":{},"cookies":[],"body":"foo","isBase64Encoded":false}"#
        );
    }

    #[test]
    fn serialize_multi_value_headers() {
        let res = LambdaResponse::from_response(
            &RequestOrigin::ApiGateway,
            Response::builder()
                .header("multi", "a")
                .header("multi", "b")
                .body(Body::from(()))
                .expect("failed to create response"),
        );
        let json = serde_json::to_string(&res).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"multi":"a"},"multiValueHeaders":{"multi":["a","b"]},"isBase64Encoded":false}"#
        )
    }

    #[test]
    fn serialize_cookies() {
        let res = LambdaResponse::from_response(
            &RequestOrigin::ApiGatewayV2,
            Response::builder()
                .header("set-cookie", "cookie1=a")
                .header("set-cookie", "cookie2=b")
                .body(Body::from(()))
                .expect("failed to create response"),
        );
        let json = serde_json::to_string(&res).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{},"cookies":["cookie1=a","cookie2=b"],"isBase64Encoded":false}"#
        )
    }
}
