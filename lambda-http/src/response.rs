//! Response types

use crate::request::RequestOrigin;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::alb::AlbTargetGroupResponse;
use aws_lambda_events::event::apigw::{ApiGatewayProxyResponse, ApiGatewayV2httpResponse};
use http::{
    header::{CONTENT_TYPE, SET_COOKIE},
    Response,
};
use http_body::Body as HttpBody;
use hyper::body::to_bytes;
use serde::Serialize;
use std::future::ready;
use std::{
    any::{Any, TypeId},
    pin::Pin,
    future::Future,
};

/// Representation of Lambda response
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum LambdaResponse {
    ApiGatewayV2(ApiGatewayV2httpResponse),
    ApiGatewayV1(ApiGatewayProxyResponse),
    Alb(AlbTargetGroupResponse),
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
                let cookies = headers
                    .get_all(SET_COOKIE)
                    .iter()
                    .cloned()
                    .map(|v| v.to_str().ok().unwrap_or_default().to_string())
                    .collect();
                headers.remove(SET_COOKIE);

                LambdaResponse::ApiGatewayV2(ApiGatewayV2httpResponse {
                    body,
                    status_code: status_code as i64,
                    is_base64_encoded: Some(is_base64_encoded),
                    cookies,
                    headers: headers.clone(),
                    multi_value_headers: headers,
                })
            }
            RequestOrigin::ApiGatewayV1 => LambdaResponse::ApiGatewayV1(ApiGatewayProxyResponse {
                body,
                status_code: status_code as i64,
                is_base64_encoded: Some(is_base64_encoded),
                headers: headers.clone(),
                multi_value_headers: headers,
            }),
            RequestOrigin::Alb => LambdaResponse::Alb(AlbTargetGroupResponse {
                body,
                status_code: status_code as i64,
                is_base64_encoded,
                headers: headers.clone(),
                multi_value_headers: headers,
                status_description: Some(format!(
                    "{} {}",
                    status_code,
                    parts.status.canonical_reason().unwrap_or_default()
                )),
            }),
            RequestOrigin::WebSocket => LambdaResponse::ApiGatewayV1(ApiGatewayProxyResponse {
                body,
                status_code: status_code as i64,
                is_base64_encoded: Some(is_base64_encoded),
                headers: headers.clone(),
                multi_value_headers: headers,
            }),
        }
    }
}

pub trait IntoResponse {
    fn into_response(self) -> ResponseFuture;
}

impl<B> IntoResponse for Response<B>
where
    B: IntoBody + 'static,
{
    fn into_response(self) -> ResponseFuture {
        let (parts, body) = self.into_parts();

        let fut = async {
            Response::from_parts(parts, body.into_body().await)
        };

        Box::pin(fut)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> ResponseFuture {
        Box::pin(async move {
            Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_string(&self)
                    .expect("unable to serialize serde_json::Value")
                    .into(),
            )
            .expect("unable to build http::Response")
        })
    }
}

pub type ResponseFuture = Pin<Box<dyn Future<Output=Response<Body>>>>;


pub trait IntoBody {
    fn into_body(self) -> BodyFuture;
}

impl<B> IntoBody for B
where
    B: HttpBody + Unpin + 'static,
    B::Error: std::fmt::Debug,
{
    fn into_body(self) -> BodyFuture {
        if TypeId::of::<Body>() == self.type_id() {
            let any_self = Box::new(self) as Box<dyn Any + 'static>;
            // Can safely unwrap here as we do type validation in the 'if' statement
            Box::pin(ready(*any_self.downcast::<Body>().unwrap()))
        } else {
            Box::pin(async move {
                Body::from(to_bytes(self).await.expect("unable to read bytes from body").to_vec())
            })
        }
    }
}

pub type BodyFuture = Pin<Box<dyn Future<Output=Body>>>;

#[cfg(test)]
mod tests {
    use super::{Body, IntoResponse, LambdaResponse, RequestOrigin};
    use http::{header::CONTENT_TYPE, Response};
    use serde_json::{self, json};

    #[tokio::test]
    async fn json_into_response() {
        let response = json!({ "hello": "lambda"}).into_response().await;
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

    #[tokio::test]
    async fn text_into_response() {
        let response = "text".into_response().await;
        match response.body() {
            Body::Text(text) => assert_eq!(text, "text"),
            _ => panic!("invalid body"),
        }
    }

    #[test]
    fn serialize_multi_value_headers() {
        let res = LambdaResponse::from_response(
            &RequestOrigin::ApiGatewayV1,
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
            "{\"statusCode\":200,\"headers\":{},\"multiValueHeaders\":{},\"isBase64Encoded\":false,\"cookies\":[\"cookie1=a\",\"cookie2=b\"]}",
            json
        )
    }
}
