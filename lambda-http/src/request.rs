//! ALB and API Gateway request adaptations
//!
//! Typically these are exposed via the `request_context`
//! request extension method provided by [lambda_http::RequestExt](../trait.RequestExt.html)
//!
use crate::{
    body::Body,
    ext::{PathParameters, QueryStringParameters, StageVariables},
    strmap::StrMap,
};
use serde::{
    de::{Deserializer, Error as DeError, MapAccess, Visitor},
    Deserialize,
};
use serde_json::{error::Error as JsonError, Value};
use std::{borrow::Cow, collections::HashMap, fmt, io::Read, mem};

/// Internal representation of an Lambda http event from
/// ALB, API Gateway REST and HTTP API proxy event perspectives
///
/// This is not intended to be a type consumed by crate users directly. The order
/// of the variants are notable. Serde will try to deserialize in this order.
#[doc(hidden)]
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LambdaRequest<'a> {
    #[serde(rename_all = "camelCase")]
    ApiGatewayV2 {
        version: Cow<'a, str>,
        route_key: Cow<'a, str>,
        raw_path: Cow<'a, str>,
        raw_query_string: Cow<'a, str>,
        cookies: Option<Vec<Cow<'a, str>>>,
        #[serde(deserialize_with = "deserialize_headers")]
        headers: http::HeaderMap,
        #[serde(default)]
        query_string_parameters: StrMap,
        #[serde(default)]
        path_parameters: StrMap,
        #[serde(default)]
        stage_variables: StrMap,
        body: Option<Cow<'a, str>>,
        #[serde(default)]
        is_base64_encoded: bool,
        request_context: ApiGatewayV2RequestContext,
    },
    #[serde(rename_all = "camelCase")]
    Alb {
        path: Cow<'a, str>,
        #[serde(deserialize_with = "deserialize_method")]
        http_method: http::Method,
        #[serde(deserialize_with = "deserialize_headers")]
        headers: http::HeaderMap,
        /// For alb events these are only present when
        /// the `lambda.multi_value_headers.enabled` target group setting turned on
        #[serde(default, deserialize_with = "deserialize_multi_value_headers")]
        multi_value_headers: http::HeaderMap,
        #[serde(deserialize_with = "nullable_default")]
        query_string_parameters: StrMap,
        /// For alb events these are only present when
        /// the `lambda.multi_value_headers.enabled` target group setting turned on
        #[serde(default, deserialize_with = "nullable_default")]
        multi_value_query_string_parameters: StrMap,
        body: Option<Cow<'a, str>>,
        #[serde(default)]
        is_base64_encoded: bool,
        request_context: AlbRequestContext,
    },
    #[serde(rename_all = "camelCase")]
    ApiGateway {
        path: Cow<'a, str>,
        #[serde(deserialize_with = "deserialize_method")]
        http_method: http::Method,
        #[serde(deserialize_with = "deserialize_headers")]
        headers: http::HeaderMap,
        #[serde(default, deserialize_with = "deserialize_multi_value_headers")]
        multi_value_headers: http::HeaderMap,
        #[serde(deserialize_with = "nullable_default")]
        query_string_parameters: StrMap,
        #[serde(default, deserialize_with = "nullable_default")]
        multi_value_query_string_parameters: StrMap,
        #[serde(default, deserialize_with = "nullable_default")]
        path_parameters: StrMap,
        #[serde(default, deserialize_with = "nullable_default")]
        stage_variables: StrMap,
        body: Option<Cow<'a, str>>,
        #[serde(default)]
        is_base64_encoded: bool,
        request_context: ApiGatewayRequestContext,
    },
}

impl LambdaRequest<'_> {
    /// Return true if this request represents an ALB event
    ///
    /// Alb responses have unique requirements for responses that
    /// vary only slightly from APIGateway responses. We serialize
    /// responses capturing a hint that the request was an alb triggered
    /// event.
    pub fn is_alb(&self) -> bool {
        matches!(self, LambdaRequest::Alb { .. })
    }
}

/// See [context-variable-reference](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-mapping-template-reference.html) for more detail.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2RequestContext {
    /// The API owner's AWS account ID.
    pub account_id: String,
    /// The identifier API Gateway assigns to your API.
    pub api_id: String,
    /// The stringified value of the specified key-value pair of the context map returned from an API Gateway Lambda authorizer function.
    #[serde(default)]
    pub authorizer: HashMap<String, Value>,
    /// The full domain name used to invoke the API. This should be the same as the incoming Host header.
    pub domain_name: String,
    /// The first label of the $context.domainName. This is often used as a caller/customer identifier.
    pub domain_prefix: String,
    /// The HTTP method used.
    pub http: Http,
    /// The ID that API Gateway assigns to the API request.
    pub request_id: String,
    /// Undocumented, could be resourcePath
    pub route_key: String,
    /// The deployment stage of the API request (for example, Beta or Prod).
    pub stage: String,
    /// Undocumented, could be requestTime
    pub time: String,
    /// Undocumented, could be requestTimeEpoch
    pub time_epoch: usize,
}

/// See [context-variable-reference](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-mapping-template-reference.html) for more detail.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestContext {
    /// The API owner's AWS account ID.
    pub account_id: String,
    /// The identifier that API Gateway assigns to your resource.
    pub resource_id: String,
    /// The deployment stage of the API request (for example, Beta or Prod).
    pub stage: String,
    /// The ID that API Gateway assigns to the API request.
    pub request_id: String,
    /// The path to your resource. For example, for the non-proxy request URI of `https://{rest-api-id.execute-api.{region}.amazonaws.com/{stage}/root/child`, The $context.resourcePath value is /root/child.
    pub resource_path: String,
    /// The HTTP method used. Valid values include: DELETE, GET, HEAD, OPTIONS, PATCH, POST, and PUT.
    pub http_method: String,
    /// The stringified value of the specified key-value pair of the context map returned from an API Gateway Lambda authorizer function.
    #[serde(default)]
    pub authorizer: HashMap<String, Value>,
    /// The identifier API Gateway assigns to your API.
    pub api_id: String,
    /// Cofnito identity information
    pub identity: Identity,
}

/// Elastic load balancer context information
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlbRequestContext {
    /// Elastic load balancer context information
    pub elb: Elb,
}

/// Event request context as an enumeration of request contexts
/// for both ALB and API Gateway and HTTP API events
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestContext {
    /// API Gateway v2 request context
    ApiGatewayV2(ApiGatewayV2RequestContext),
    /// API Gateway request context
    ApiGateway(ApiGatewayRequestContext),
    /// ALB request context
    Alb(AlbRequestContext),
}

/// Elastic load balancer context information
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Elb {
    /// AWS ARN identifier for the ELB Target Group this lambda was triggered by
    pub target_group_arn: String,
}

/// Http information captured API Gateway v2 request context
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    #[serde(deserialize_with = "deserialize_method")]
    /// The HTTP method used. Valid values include: DELETE, GET, HEAD, OPTIONS, PATCH, POST, and PUT.
    pub method: http::Method,
    /// The request path. For example, for a non-proxy request URL of
    /// `https://{rest-api-id.execute-api.{region}.amazonaws.com/{stage}/root/child`,
    /// the $context.path value is `/{stage}/root/child`.
    pub path: String,
    /// The request protocol, for example, HTTP/1.1.
    pub protocol: String,
    /// The source IP address of the TCP connection making the request to API Gateway.
    pub source_ip: String,
    /// The User-Agent header of the API caller.
    pub user_agent: String,
}

/// Identity assoicated with API Gateway request
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    /// The source IP address of the TCP connection making the request to API Gateway.
    pub source_ip: String,
    /// The Amazon Cognito identity ID of the caller making the request.
    /// Available only if the request was signed with Amazon Cognito credentials.
    pub cognito_identity_id: Option<String>,
    /// The Amazon Cognito identity pool ID of the caller making the request.
    /// Available only if the request was signed with Amazon Cognito credentials.
    pub cognito_identity_pool_id: Option<String>,
    /// A comma-separated list of the Amazon Cognito authentication providers used by the caller making the request.
    /// Available only if the request was signed with Amazon Cognito credentials.
    pub cognito_authentication_provider: Option<String>,
    /// The Amazon Cognito authentication type of the caller making the request.
    /// Available only if the request was signed with Amazon Cognito credentials.
    pub cognito_authentication_type: Option<String>,
    /// The AWS account ID associated with the request.
    pub account_id: Option<String>,
    /// The principal identifier of the caller making the request.
    pub caller: Option<String>,
    /// For API methods that require an API key, this variable is the API key associated with the method request.
    /// For methods that don't require an API key, this variable is null.
    pub api_key: Option<String>,
    /// Undocumented. Can be the API key ID associated with an API request that requires an API key.
    /// The description of `api_key` and `access_key` may actually be reversed.
    pub access_key: Option<String>,
    /// The principal identifier of the user making the request. Used in Lambda authorizers.
    pub user: Option<String>,
    /// The User-Agent header of the API caller.
    pub user_agent: Option<String>,
    /// The Amazon Resource Name (ARN) of the effective user identified after authentication.
    pub user_arn: Option<String>,
}

/// Deserialize a str into an http::Method
fn deserialize_method<'de, D>(deserializer: D) -> Result<http::Method, D::Error>
where
    D: Deserializer<'de>,
{
    struct MethodVisitor;

    impl<'de> Visitor<'de> for MethodVisitor {
        type Value = http::Method;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a Method")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            v.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_str(MethodVisitor)
}

/// Deserialize a map of Cow<'_, str> => Vec<Cow<'_, str>> into an http::HeaderMap
fn deserialize_multi_value_headers<'de, D>(deserializer: D) -> Result<http::HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct HeaderVisitor;

    impl<'de> Visitor<'de> for HeaderVisitor {
        type Value = http::HeaderMap;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a multi valued HeaderMap<HeaderValue>")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut headers = map
                .size_hint()
                .map(http::HeaderMap::with_capacity)
                .unwrap_or_else(http::HeaderMap::new);
            while let Some((key, values)) = map.next_entry::<Cow<'_, str>, Vec<Cow<'_, str>>>()? {
                // note the aws docs for multi value headers include an empty key. I'm not sure if this is a doc bug
                // or not by the http crate doesn't handle it
                // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
                if !key.is_empty() {
                    for value in values {
                        let header_name = key.parse::<http::header::HeaderName>().map_err(A::Error::custom)?;
                        let header_value = http::header::HeaderValue::from_maybe_shared(value.into_owned())
                            .map_err(A::Error::custom)?;
                        headers.append(header_name, header_value);
                    }
                }
            }
            Ok(headers)
        }
    }

    deserializer.deserialize_map(HeaderVisitor)
}

/// Deserialize a map of Cow<'_, str> => Cow<'_, str> into an http::HeaderMap
fn deserialize_headers<'de, D>(deserializer: D) -> Result<http::HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct HeaderVisitor;

    impl<'de> Visitor<'de> for HeaderVisitor {
        type Value = http::HeaderMap;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a HeaderMap<HeaderValue>")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut headers = map
                .size_hint()
                .map(http::HeaderMap::with_capacity)
                .unwrap_or_else(http::HeaderMap::new);
            while let Some((key, value)) = map.next_entry::<Cow<'_, str>, Cow<'_, str>>()? {
                let header_name = key.parse::<http::header::HeaderName>().map_err(A::Error::custom)?;
                let header_value =
                    http::header::HeaderValue::from_maybe_shared(value.into_owned()).map_err(A::Error::custom)?;
                headers.append(header_name, header_value);
            }
            Ok(headers)
        }
    }

    deserializer.deserialize_map(HeaderVisitor)
}

/// deserializes (json) null values to their default values
// https://github.com/serde-rs/serde/issues/1098
fn nullable_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(T::default))
}

/// Converts LambdaRequest types into `http::Request<Body>` types
impl<'a> From<LambdaRequest<'a>> for http::Request<Body> {
    fn from(value: LambdaRequest<'_>) -> Self {
        match value {
            LambdaRequest::ApiGatewayV2 {
                raw_path,
                raw_query_string,
                mut headers,
                query_string_parameters,
                path_parameters,
                stage_variables,
                body,
                is_base64_encoded,
                request_context,
                cookies,
                ..
            } => {
                if let Some(cookies) = cookies {
                    if let Ok(header_value) = http::header::HeaderValue::from_str(&cookies.join(";")) {
                        headers.append(http::header::COOKIE, header_value);
                    }
                }

                let builder = http::Request::builder()
                    .method(request_context.http.method.as_ref())
                    .uri({
                        let mut url = format!(
                            "{}://{}{}",
                            headers
                                .get("X-Forwarded-Proto")
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_else(|| "https"),
                            headers
                                .get(http::header::HOST)
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_else(|| request_context.domain_name.as_ref()),
                            raw_path
                        );
                        if !raw_query_string.is_empty() {
                            url.push('?');
                            url.push_str(raw_query_string.as_ref());
                        }
                        url
                    })
                    .extension(QueryStringParameters(query_string_parameters))
                    .extension(PathParameters(path_parameters))
                    .extension(StageVariables(stage_variables))
                    .extension(RequestContext::ApiGatewayV2(request_context));

                let mut req = builder
                    .body(body.map_or_else(Body::default, |b| Body::from_maybe_encoded(is_base64_encoded, b)))
                    .expect("failed to build request");

                // no builder method that sets headers in batch
                let _ = mem::replace(req.headers_mut(), headers);

                req
            }
            LambdaRequest::ApiGateway {
                path,
                http_method,
                headers,
                mut multi_value_headers,
                query_string_parameters,
                multi_value_query_string_parameters,
                path_parameters,
                stage_variables,
                body,
                is_base64_encoded,
                request_context,
            } => {
                let builder = http::Request::builder()
                    .method(http_method)
                    .uri({
                        format!(
                            "{}://{}{}",
                            headers
                                .get("X-Forwarded-Proto")
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_else(|| "https"),
                            headers
                                .get(http::header::HOST)
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_default(),
                            path
                        )
                    })
                    // multi-valued query string parameters are always a super
                    // set of singly valued query string parameters,
                    // when present, multi-valued query string parameters are preferred
                    .extension(QueryStringParameters(
                        if multi_value_query_string_parameters.is_empty() {
                            query_string_parameters
                        } else {
                            multi_value_query_string_parameters
                        },
                    ))
                    .extension(PathParameters(path_parameters))
                    .extension(StageVariables(stage_variables))
                    .extension(RequestContext::ApiGateway(request_context));

                let mut req = builder
                    .body(body.map_or_else(Body::default, |b| Body::from_maybe_encoded(is_base64_encoded, b)))
                    .expect("failed to build request");

                // merge headers into multi_value_headers and make
                // multi-value_headers our cannoncial source of request headers
                for (key, value) in headers {
                    // see HeaderMap#into_iter() docs for cases when key element may be None
                    if let Some(first_key) = key {
                        // if it contains the key, avoid appending a duplicate value
                        if !multi_value_headers.contains_key(&first_key) {
                            multi_value_headers.append(first_key, value);
                        }
                    }
                }

                // no builder method that sets headers in batch
                let _ = mem::replace(req.headers_mut(), multi_value_headers);

                req
            }
            LambdaRequest::Alb {
                path,
                http_method,
                headers,
                mut multi_value_headers,
                query_string_parameters,
                multi_value_query_string_parameters,
                body,
                is_base64_encoded,
                request_context,
            } => {
                // build an http::Request<lambda_http::Body> from a lambda_http::LambdaRequest
                let builder = http::Request::builder()
                    .method(http_method)
                    .uri({
                        format!(
                            "{}://{}{}",
                            headers
                                .get("X-Forwarded-Proto")
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_else(|| "https"),
                            headers
                                .get(http::header::HOST)
                                .and_then(|val| val.to_str().ok())
                                .unwrap_or_default(),
                            path
                        )
                    })
                    // multi valued query string parameters are always a super
                    // set of singly valued query string parameters,
                    // when present, multi-valued query string parameters are preferred
                    .extension(QueryStringParameters(
                        if multi_value_query_string_parameters.is_empty() {
                            query_string_parameters
                        } else {
                            multi_value_query_string_parameters
                        },
                    ))
                    .extension(RequestContext::Alb(request_context));

                let mut req = builder
                    .body(body.map_or_else(Body::default, |b| Body::from_maybe_encoded(is_base64_encoded, b)))
                    .expect("failed to build request");

                // merge headers into multi_value_headers and make
                // multi-value_headers our cannoncial source of request headers
                for (key, value) in headers {
                    // see HeaderMap#into_iter() docs for cases when key element may be None
                    if let Some(first_key) = key {
                        // if it contains the key, avoid appending a duplicate value
                        if !multi_value_headers.contains_key(&first_key) {
                            multi_value_headers.append(first_key, value);
                        }
                    }
                }

                // no builder method that sets headers in batch
                let _ = mem::replace(req.headers_mut(), multi_value_headers);

                req
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestExt;
    use serde_json;
    use std::{collections::HashMap, fs::File};

    #[test]
    fn deserializes_apigw_request_events_from_readables() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        // note: file paths are relative to the directory of the crate at runtime
        let result = from_reader(File::open("tests/data/apigw_proxy_request.json").expect("expected file"));
        assert!(result.is_ok(), format!("event was not parsed as expected {:?}", result));
    }

    #[test]
    fn deserializes_minimal_apigw_v2_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request_minimal.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            format!("event was not parsed as expected {:?} given {}", result, input)
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "https://xxx.execute-api.us-east-1.amazonaws.com/");
    }

    #[test]
    fn deserializes_apigw_v2_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            format!("event was not parsed as expected {:?} given {}", result, input)
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
            format!("event was not parsed as expected {:?} given {}", result, input)
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://wt6mne2s9k.execute-api.us-west-2.amazonaws.com/test/hello"
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
            format!("event was not parsed as expected {:?} given {}", result, input)
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "https://lambda-846800462-us-east-2.elb.amazonaws.com/");
    }

    #[test]
    fn deserializes_apigw_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/apigw_multi_value_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            format!("event is was not parsed as expected {:?} given {}", result, input)
        );
        let request = result.expect("failed to parse request");

        assert!(!request.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            request.query_string_parameters().get_all("multivalueName"),
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
            format!("event is was not parsed as expected {:?} given {}", result, input)
        );
        let request = result.expect("failed to parse request");
        assert!(!request.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            request.query_string_parameters().get_all("myKey"),
            Some(vec!["val1", "val2"])
        );
    }

    #[test]
    fn deserialize_with_null() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Test {
            #[serde(deserialize_with = "nullable_default")]
            foo: HashMap<String, String>,
        }

        assert_eq!(
            serde_json::from_str::<Test>(r#"{"foo":null}"#).expect("failed to deserialize"),
            Test { foo: HashMap::new() }
        )
    }
}
