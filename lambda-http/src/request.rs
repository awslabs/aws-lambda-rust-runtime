//! ALB and API Gateway request adaptations
//!
//! Typically these are exposed via the `request_context`
//! request extension method provided by [lambda_http::RequestExt](../trait.RequestExt.html)
//!
use crate::ext::{PathParameters, QueryStringParameters, StageVariables};
use aws_lambda_events::alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext};
use aws_lambda_events::apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContext,
    ApiGatewayWebsocketProxyRequest, ApiGatewayWebsocketProxyRequestContext,
};
use aws_lambda_events::encodings::Body;
use http::header::HeaderName;
use query_map::QueryMap;
use serde::Deserialize;
use serde_json::error::Error as JsonError;
use std::{io::Read, mem};

/// Internal representation of an Lambda http event from
/// ALB, API Gateway REST and HTTP API proxy event perspectives
///
/// This is not intended to be a type consumed by crate users directly. The order
/// of the variants are notable. Serde will try to deserialize in this order.
#[doc(hidden)]
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LambdaRequest {
    ApiGatewayV1(ApiGatewayProxyRequest),
    ApiGatewayV2(ApiGatewayV2httpRequest),
    Alb(AlbTargetGroupRequest),
    WebSocket(ApiGatewayWebsocketProxyRequest),
}

impl LambdaRequest {
    /// Return the `RequestOrigin` of the request to determine where the `LambdaRequest`
    /// originated from, so that the appropriate response can be selected based on what
    /// type of response the request origin expects.
    pub fn request_origin(&self) -> RequestOrigin {
        match self {
            LambdaRequest::ApiGatewayV1 { .. } => RequestOrigin::ApiGatewayV1,
            LambdaRequest::ApiGatewayV2 { .. } => RequestOrigin::ApiGatewayV2,
            LambdaRequest::Alb { .. } => RequestOrigin::Alb,
            LambdaRequest::WebSocket { .. } => RequestOrigin::WebSocket,
        }
    }
}

/// Represents the origin from which the lambda was requested from.
#[doc(hidden)]
#[derive(Debug)]
pub enum RequestOrigin {
    /// API Gateway request origin
    ApiGatewayV1,
    /// API Gateway v2 request origin
    ApiGatewayV2,
    /// ALB request origin
    Alb,
    /// API Gateway WebSocket
    WebSocket,
}

fn into_api_gateway_v2_request(ag: ApiGatewayV2httpRequest) -> http::Request<Body> {
    let http_method = ag.request_context.http.method.clone();
    let builder = http::Request::builder()
        .uri({
            let scheme = ag
                .headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let host = ag
                .headers
                .get(http::header::HOST)
                .and_then(|s| s.to_str().ok())
                .or_else(|| ag.request_context.domain_name.as_deref())
                .unwrap_or_default();

            let path = apigw_path_with_stage(&ag.request_context.stage, ag.raw_path.as_deref().unwrap_or_default());
            let mut url = format!("{}://{}{}", scheme, host, path);

            if let Some(query) = ag.raw_query_string {
                url.push('?');
                url.push_str(&query);
            }
            url
        })
        .extension(QueryStringParameters(ag.query_string_parameters))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV2(ag.request_context));

    let mut headers = ag.headers;
    if let Some(cookies) = ag.cookies {
        if let Ok(header_value) = http::header::HeaderValue::from_str(&cookies.join(";")) {
            headers.append(http::header::COOKIE, header_value);
        }
    }

    let base64 = ag.is_base64_encoded;

    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}

fn into_proxy_request(ag: ApiGatewayProxyRequest) -> http::Request<Body> {
    let http_method = ag.http_method;
    let builder = http::Request::builder()
        .uri({
            let host = ag.headers.get(http::header::HOST).and_then(|s| s.to_str().ok());
            let path = apigw_path_with_stage(&ag.request_context.stage, &ag.path.unwrap_or_default());

            let mut url = match host {
                None => path,
                Some(host) => {
                    let scheme = ag
                        .headers
                        .get(x_forwarded_proto())
                        .and_then(|s| s.to_str().ok())
                        .unwrap_or("https");
                    format!("{}://{}{}", scheme, host, path)
                }
            };

            if !ag.multi_value_query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&ag.multi_value_query_string_parameters.to_query_string());
            } else if !ag.query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&ag.query_string_parameters.to_query_string());
            }
            url
        })
        // multi-valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if ag.multi_value_query_string_parameters.is_empty() {
                ag.query_string_parameters
            } else {
                ag.multi_value_query_string_parameters
            },
        ))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV1(ag.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = ag.multi_value_headers;
    headers.extend(ag.headers);

    let base64 = ag.is_base64_encoded.unwrap_or_default();
    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}

fn into_alb_request(alb: AlbTargetGroupRequest) -> http::Request<Body> {
    let http_method = alb.http_method;
    let builder = http::Request::builder()
        .uri({
            let scheme = alb
                .headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let host = alb
                .headers
                .get(http::header::HOST)
                .and_then(|s| s.to_str().ok())
                .unwrap_or_default();

            let mut url = format!("{}://{}{}", scheme, host, alb.path.unwrap_or_default());
            if !alb.multi_value_query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&alb.multi_value_query_string_parameters.to_query_string());
            } else if !alb.query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&alb.query_string_parameters.to_query_string());
            }

            url
        })
        // multi valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if alb.multi_value_query_string_parameters.is_empty() {
                alb.query_string_parameters
            } else {
                alb.multi_value_query_string_parameters
            },
        ))
        .extension(RequestContext::Alb(alb.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = alb.multi_value_headers;
    headers.extend(alb.headers);

    let base64 = alb.is_base64_encoded;

    let mut req = builder
        .body(
            alb.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}

fn into_websocket_request(ag: ApiGatewayWebsocketProxyRequest) -> http::Request<Body> {
    let http_method = ag.http_method;
    let builder = http::Request::builder()
        .uri({
            let host = ag.headers.get(http::header::HOST).and_then(|s| s.to_str().ok());
            let path = apigw_path_with_stage(&ag.request_context.stage, &ag.path.unwrap_or_default());

            let mut url = match host {
                None => path,
                Some(host) => {
                    let scheme = ag
                        .headers
                        .get(x_forwarded_proto())
                        .and_then(|s| s.to_str().ok())
                        .unwrap_or("https");
                    format!("{}://{}{}", scheme, host, path)
                }
            };

            if !ag.multi_value_query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&ag.multi_value_query_string_parameters.to_query_string());
            } else if !ag.query_string_parameters.is_empty() {
                url.push('?');
                url.push_str(&ag.query_string_parameters.to_query_string());
            }
            url
        })
        // multi-valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if ag.multi_value_query_string_parameters.is_empty() {
                ag.query_string_parameters
            } else {
                ag.multi_value_query_string_parameters
            },
        ))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::WebSocket(ag.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = ag.multi_value_headers;
    headers.extend(ag.headers);

    let base64 = ag.is_base64_encoded.unwrap_or_default();
    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method.unwrap_or(http::Method::GET));

    req
}

fn apigw_path_with_stage(stage: &Option<String>, path: &str) -> String {
    match stage {
        None => path.into(),
        Some(stage) if stage == "$default" => path.into(),
        Some(stage) => format!("/{}{}", stage, path),
    }
}

/// Event request context as an enumeration of request contexts
/// for both ALB and API Gateway and HTTP API events
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestContext {
    /// API Gateway proxy request context
    ApiGatewayV1(ApiGatewayProxyRequestContext),
    /// API Gateway v2 request context
    ApiGatewayV2(ApiGatewayV2httpRequestContext),
    /// ALB request context
    Alb(AlbTargetGroupRequestContext),
    /// WebSocket request context
    WebSocket(ApiGatewayWebsocketProxyRequestContext),
}

/// Converts LambdaRequest types into `http::Request<Body>` types
impl<'a> From<LambdaRequest> for http::Request<Body> {
    fn from(value: LambdaRequest) -> Self {
        match value {
            LambdaRequest::ApiGatewayV2(ag) => into_api_gateway_v2_request(ag),
            LambdaRequest::ApiGatewayV1(ag) => into_proxy_request(ag),
            LambdaRequest::Alb(alb) => into_alb_request(alb),
            LambdaRequest::WebSocket(ag) => into_websocket_request(ag),
        }
    }
}

/// Deserializes a `Request` from a `Read` impl providing JSON events.
///
/// # Example
///
/// ```rust,no_run
/// use lambda_http::request::from_reader;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_reader(
///         File::open("path/to/request.json")?
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_reader<R>(rdr: R) -> Result<crate::Request, JsonError>
where
    R: Read,
{
    serde_json::from_reader(rdr).map(LambdaRequest::into)
}

/// Deserializes a `Request` from a string of JSON text.
///
/// # Example
///
/// ```rust,no_run
/// use lambda_http::request::from_str;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_str(
///         r#"{ ...raw json here... }"#
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_str(s: &str) -> Result<crate::Request, JsonError> {
    serde_json::from_str(s).map(LambdaRequest::into)
}

fn x_forwarded_proto() -> HeaderName {
    HeaderName::from_static("x-forwarded-proto")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestExt;
    use std::fs::File;

    #[test]
    fn deserializes_apigw_request_events_from_readables() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        // note: file paths are relative to the directory of the crate at runtime
        let result = from_reader(File::open("tests/data/apigw_proxy_request.json").expect("expected file"));
        assert!(result.is_ok(), "event was not parsed as expected {:?}", result);
    }

    #[test]
    fn deserializes_minimal_apigw_v2_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request_minimal.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "https://xxx.execute-api.us-east-1.amazonaws.com/");

        // Ensure this is an APIGWv2 request
        let req_context = req.request_context();
        assert!(
            match req_context {
                RequestContext::ApiGatewayV2(_) => true,
                _ => false,
            },
            "expected ApiGatewayV2 context, got {:?}",
            req_context
        );
    }

    #[test]
    fn deserializes_apigw_v2_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        let cookie_header = req
            .headers()
            .get(http::header::COOKIE)
            .ok_or_else(|| "Cookie header not found".to_string())
            .and_then(|v| v.to_str().map_err(|e| e.to_string()));

        assert_eq!(req.method(), "POST");
        assert_eq!(req.uri(), "https://id.execute-api.us-east-1.amazonaws.com/my/path?parameter1=value1&parameter1=value2&parameter2=value");
        assert_eq!(cookie_header, Ok("cookie1=value1;cookie2=value2"));

        // Ensure this is an APIGWv2 request
        let req_context = req.request_context();
        assert!(
            match req_context {
                RequestContext::ApiGatewayV2(_) => true,
                _ => false,
            },
            "expected ApiGatewayV2 context, got {:?}",
            req_context
        );
    }

    #[test]
    fn deserializes_apigw_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
        let input = include_str!("../tests/data/apigw_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://wt6mne2s9k.execute-api.us-west-2.amazonaws.com/test/test/hello?name=me"
        );

        // Ensure this is an APIGW request
        let req_context = req.request_context();
        assert!(
            match req_context {
                RequestContext::ApiGatewayV1(_) => true,
                _ => false,
            },
            "expected ApiGateway context, got {:?}",
            req_context
        );
    }

    #[test]
    fn deserializes_alb_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html#multi-value-headers
        let input = include_str!("../tests/data/alb_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://lambda-846800462-us-east-2.elb.amazonaws.com/?myKey=val2"
        );

        // Ensure this is an ALB request
        let req_context = req.request_context();
        assert!(
            match req_context {
                RequestContext::Alb(_) => true,
                _ => false,
            },
            "expected Alb context, got {:?}",
            req_context
        );
    }

    #[test]
    fn deserializes_apigw_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/apigw_multi_value_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event is was not parsed as expected {:?} given {}",
            result,
            input
        );
        let request = result.expect("failed to parse request");

        assert!(!request.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            request.query_string_parameters().all("multivalueName"),
            Some(vec!["you", "me"])
        );
    }

    #[test]
    fn deserializes_alb_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/alb_multi_value_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event is was not parsed as expected {:?} given {}",
            result,
            input
        );
        let request = result.expect("failed to parse request");
        assert!(!request.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            request.query_string_parameters().all("myKey"),
            Some(vec!["val1", "val2"])
        );
    }

    #[test]
    fn deserialize_apigw_v2_sam_local() {
        // manually generated from AWS SAM CLI
        // Steps to recreate:
        // * sam init
        // * Use, Zip Python 3.9, and Hello World example
        // * Change the template to use HttpApi instead of Api
        // * Change the function code to return the Lambda event serialized
        // * sam local start-api
        // * Invoke the API
        let input = include_str!("../tests/data/apigw_v2_sam_local.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "http://127.0.0.1:3000/hello");
    }

    #[test]
    fn deserialize_apigw_no_host() {
        // generated from the 'apigateway-aws-proxy' test event template in the Lambda console
        let input = include_str!("../tests/data/apigw_no_host.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {:?} given {}",
            result,
            input
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "/test/test/hello?name=me");
    }
}
