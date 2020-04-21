//! ALB and API Gateway request types.
//!
//! Typically these are exposed via the `request_context`
//! request extension method provided by [lambda_http::RequestExt](../trait.RequestExt.html)
//!
use http::{
    self,
    header::{HeaderName, HeaderValue, HOST},
    HeaderMap, Method, Request as HttpRequest,
};
use serde::de::{Deserialize, Deserializer, Error as DeError, MapAccess, Visitor};
use serde_derive::Deserialize;
use serde_json::{error::Error as JsonError, Value};
use std::{borrow::Cow, collections::HashMap, fmt, io::Read, mem};

use crate::{
    body::Body,
    ext::{PathParameters, QueryStringParameters, StageVariables},
    strmap::StrMap,
};

/// Internal representation of an Lambda http event from both
/// both ALB and API Gateway proxy event perspectives
#[doc(hidden)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LambdaRequest<'a> {
    pub(crate) path: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_method")]
    pub(crate) http_method: Method,
    #[serde(deserialize_with = "deserialize_headers")]
    pub(crate) headers: HeaderMap<HeaderValue>,
    /// For alb events these are only present when
    /// the `lambda.multi_value_headers.enabled` target group setting turned on
    #[serde(default, deserialize_with = "deserialize_multi_value_headers")]
    pub(crate) multi_value_headers: HeaderMap<HeaderValue>,
    #[serde(deserialize_with = "nullable_default")]
    pub(crate) query_string_parameters: StrMap,
    /// For alb events these are only present when
    /// the `lambda.multi_value_headers.enabled` target group setting turned on
    #[serde(default, deserialize_with = "nullable_default")]
    pub(crate) multi_value_query_string_parameters: StrMap,
    /// ALB events do not have path parameters.
    #[serde(default, deserialize_with = "nullable_default")]
    pub(crate) path_parameters: StrMap,
    /// ALB events do not have stage variables.
    #[serde(default, deserialize_with = "nullable_default")]
    pub(crate) stage_variables: StrMap,
    pub(crate) body: Option<Cow<'a, str>>,
    #[serde(default)]
    pub(crate) is_base64_encoded: bool,
    pub(crate) request_context: RequestContext,
}

/// Event request context as an enumeration of request contexts
/// for both ALB and API Gateway http events
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestContext {
    /// Api Gateway request context
    #[serde(rename_all = "camelCase")]
    ApiGateway {
        //pub path: String,
        account_id: String,
        resource_id: String,
        stage: String,
        request_id: String,
        resource_path: String,
        http_method: String,
        #[serde(default)]
        authorizer: HashMap<String, Value>,
        api_id: String,
        identity: Identity,
    },
    /// ALB request context
    #[serde(rename_all = "camelCase")]
    Alb { elb: Elb },
}

impl Default for RequestContext {
    fn default() -> Self {
        RequestContext::ApiGateway {
            account_id: Default::default(),
            resource_id: Default::default(),
            stage: Default::default(),
            request_id: Default::default(),
            resource_path: Default::default(),
            http_method: Default::default(),
            authorizer: Default::default(),
            api_id: Default::default(),
            identity: Default::default(),
        }
    }
}

impl RequestContext {
    /// Return true if this request context represents an ALB request
    pub fn is_alb(&self) -> bool {
        match self {
            RequestContext::Alb { .. } => true,
            _ => false,
        }
    }
}

/// Elastic load balancer context information
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Elb {
    /// AWS ARN identifier for the ELB Target Group this lambda was triggered by
    pub target_group_arn: String,
}

/// Identity assoicated with API Gateway request
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub source_ip: String,
    pub cognito_identity_id: Option<String>,
    pub cognito_identity_pool_id: Option<String>,
    pub cognito_authentication_provider: Option<String>,
    pub cognito_authentication_type: Option<String>,
    pub account_id: Option<String>,
    pub caller: Option<String>,
    pub api_key: Option<String>,
    pub access_key: Option<String>,
    pub user: Option<String>,
    pub user_agent: Option<String>,
    pub user_arn: Option<String>,
}

/// Deserialize a str into an http::Method
fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    struct MethodVisitor;

    impl<'de> Visitor<'de> for MethodVisitor {
        type Value = Method;

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
fn deserialize_multi_value_headers<'de, D>(deserializer: D) -> Result<HeaderMap<HeaderValue>, D::Error>
where
    D: Deserializer<'de>,
{
    struct HeaderVisitor;

    impl<'de> Visitor<'de> for HeaderVisitor {
        type Value = HeaderMap<HeaderValue>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a multi valued HeaderMap<HeaderValue>")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut headers = map
                .size_hint()
                .map(HeaderMap::with_capacity)
                .unwrap_or_else(HeaderMap::new);
            while let Some((key, values)) = map.next_entry::<Cow<'_, str>, Vec<Cow<'_, str>>>()? {
                // note the aws docs for multi value headers include an empty key. I'm not sure if this is a doc bug
                // or not by the http crate doesn't handle it
                // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
                if !key.is_empty() {
                    for value in values {
                        let header_name = key.parse::<HeaderName>().map_err(A::Error::custom)?;
                        let header_value =
                            HeaderValue::from_maybe_shared(value.into_owned()).map_err(A::Error::custom)?;
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
fn deserialize_headers<'de, D>(deserializer: D) -> Result<HeaderMap<HeaderValue>, D::Error>
where
    D: Deserializer<'de>,
{
    struct HeaderVisitor;

    impl<'de> Visitor<'de> for HeaderVisitor {
        type Value = HeaderMap<HeaderValue>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a HeaderMap<HeaderValue>")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut headers = map
                .size_hint()
                .map(HeaderMap::with_capacity)
                .unwrap_or_else(HeaderMap::new);
            while let Some((key, value)) = map.next_entry::<Cow<'_, str>, Cow<'_, str>>()? {
                let header_name = key.parse::<HeaderName>().map_err(A::Error::custom)?;
                let header_value = HeaderValue::from_maybe_shared(value.into_owned()).map_err(A::Error::custom)?;
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

impl<'a> From<LambdaRequest<'a>> for HttpRequest<Body> {
    fn from(value: LambdaRequest<'_>) -> Self {
        let LambdaRequest {
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
        } = value;

        // build an http::Request<lambda_http::Body> from a lambda_http::LambdaRequest
        let builder = HttpRequest::builder()
            .method(http_method)
            .uri({
                format!(
                    "{}://{}{}",
                    headers
                        .get("X-Forwarded-Proto")
                        .map(|val| val.to_str().unwrap_or_else(|_| "https"))
                        .unwrap_or_else(|| "https"),
                    headers
                        .get(HOST)
                        .map(|val| val.to_str().unwrap_or_default())
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
            .extension(PathParameters(path_parameters))
            .extension(StageVariables(stage_variables))
            .extension(request_context);

        let mut req = builder
            .body(match body {
                Some(b) => {
                    if is_base64_encoded {
                        // todo: document failure behavior
                        Body::from(::base64::decode(b.as_ref()).unwrap_or_default())
                    } else {
                        Body::from(b.into_owned())
                    }
                }
                _ => Body::from(()),
            })
            .expect("failed to build request");

        // merge headers into multi_value_headers and make
        // multi_value_headers our cannoncial source of request headers
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
        mem::replace(req.headers_mut(), multi_value_headers);

        req
    }
}

/// Deserializes a Request from an IO stream of JSON.
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

/// Deserializes a Request from a string of JSON text.
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
    fn requests_convert() {
        let mut headers = HeaderMap::new();
        headers.insert("Host", "www.rust-lang.org".parse().unwrap());
        let lambda_request: LambdaRequest<'_> = LambdaRequest {
            path: "/foo".into(),
            headers,
            ..LambdaRequest::default()
        };
        let expected = HttpRequest::get("https://www.rust-lang.org/foo").body(()).unwrap();
        let actual = HttpRequest::from(lambda_request);
        assert_eq!(expected.method(), actual.method());
        assert_eq!(expected.uri(), actual.uri());
        assert_eq!(expected.method(), actual.method());
    }

    #[test]
    fn deserializes_apigw_request_events_from_readables() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        // note: file paths are relative to the directory of the crate at runtime
        let result = from_reader(File::open("tests/data/apigw_proxy_request.json").expect("expected file"));
        assert!(result.is_ok(), format!("event was not parsed as expected {:?}", result));
    }

    #[test]
    fn deserializes_apigw_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_proxy_request.json");
        let result = from_str(input);
        assert!(result.is_ok(), format!("event was not parsed as expected {:?}", result));
    }

    #[test]
    fn deserialize_alb_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html#multi-value-headers
        let input = include_str!("../tests/data/alb_request.json");
        let result = from_str(input);
        assert!(result.is_ok(), format!("event was not parsed as expected {:?}", result));
    }

    #[test]
    fn deserializes_apigw_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/apigw_multi_value_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            format!("event is was not parsed as expected {:?}", result)
        );
        let unwrapped = result.unwrap();

        assert!(!unwrapped.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            unwrapped.query_string_parameters().get_all("multivalueName"),
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
            format!("event is was not parsed as expected {:?}", result)
        );
        let unwrapped = result.unwrap();
        assert!(!unwrapped.query_string_parameters().is_empty());

        // test RequestExt#query_string_parameters does the right thing
        assert_eq!(
            unwrapped.query_string_parameters().get_all("myKey"),
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
