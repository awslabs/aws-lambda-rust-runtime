//! Extension methods for `http::Request` types

use crate::{request::RequestContext, strmap::StrMap, Body};
use serde::{de::value::Error as SerdeError, Deserialize};
use std::{error::Error, fmt};

/// ALB/API gateway pre-parsed http query string parameters
pub(crate) struct QueryStringParameters(pub(crate) StrMap);

/// API gateway pre-extracted url path parameters
///
/// These will always be empty for ALB requests
pub(crate) struct PathParameters(pub(crate) StrMap);

/// API gateway configured
/// [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
///
/// These will always be empty for ALB requests
pub(crate) struct StageVariables(pub(crate) StrMap);

/// Request payload deserialization errors
///
/// Returned by [`RequestExt#payload()`](trait.RequestExt.html#tymethod.payload)
#[derive(Debug)]
pub enum PayloadError {
    /// Returned when `application/json` bodies fail to deserialize a payload
    Json(serde_json::Error),
    /// Returned when `application/x-www-form-urlencoded` bodies fail to deserialize a payload
    WwwFormUrlEncoded(SerdeError),
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadError::Json(json) => writeln!(f, "failed to parse payload from application/json {}", json),
            PayloadError::WwwFormUrlEncoded(form) => writeln!(
                f,
                "failed to parse payload from application/x-www-form-urlencoded {}",
                form
            ),
        }
    }
}

impl Error for PayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PayloadError::Json(json) => Some(json),
            PayloadError::WwwFormUrlEncoded(form) => Some(form),
        }
    }
}

/// Extentions for `lambda_http::Request` structs that
/// provide access to [API gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format)
/// and [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html)
/// features.
///
/// # Examples
///
/// A request's body can be deserialized if its correctly encoded as per  
/// the request's `Content-Type` header. The two supported content types are
/// `application/x-www-form-urlencoded` and `application/json`.
///
/// The following handler will work an http request body of `x=1&y=2`
/// as well as `{"x":1, "y":2}` respectively.
///
/// ```rust,no_run
/// use lambda_http::{handler, lambda::{self, Context}, Body, IntoResponse, Request, Response, RequestExt};
/// use serde::Deserialize;
///
/// type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
///
/// #[derive(Debug,Deserialize,Default)]
/// struct Args {
///   #[serde(default)]
///   x: usize,
///   #[serde(default)]
///   y: usize
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///   lambda::run(handler(add)).await?;
///   Ok(())
/// }
///
/// async fn add(
///   request: Request,
///   _: Context
/// ) -> Result<Response<Body>, Error> {
///   let args: Args = request.payload()
///     .unwrap_or_else(|_parse_err| None)
///     .unwrap_or_default();
///   Ok(
///      Response::new(
///        format!(
///          "{} + {} = {}",
///          args.x,
///          args.y,
///          args.x + args.y
///        ).into()
///      )
///   )
/// }
/// ```
pub trait RequestExt {
    /// Return pre-parsed http query string parameters, parameters
    /// provided after the `?` portion of a url,
    /// associated with the API gateway request.
    ///
    /// The yielded value represents both single and multi-valued
    /// parameters alike. When multiple query string parameters with the same
    /// name are expected, `query_string_parameters().get_all("many")` to retrieve them all.
    ///
    /// No query parameters
    /// will yield an empty `StrMap`.
    fn query_string_parameters(&self) -> StrMap;

    /// Configures instance with query string parameters under #[cfg(test)] configurations
    ///
    /// This is intended for use in mock testing contexts.
    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<StrMap>;

    /// Return pre-extracted path parameters, parameter provided in url placeholders
    /// `/foo/{bar}/baz/{boom}`,
    /// associated with the API gateway request. No path parameters
    /// will yield an empty `StrMap`
    ///
    /// These will always be empty for ALB triggered requests
    fn path_parameters(&self) -> StrMap;

    /// Configures instance with path parameters under #[cfg(test)] configurations
    ///
    /// This is intended for use in mock testing contexts.
    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<StrMap>;

    /// Return [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
    /// associated with the API gateway request. No stage parameters
    /// will yield an empty `StrMap`
    ///
    /// These will always be empty for ALB triggered requests
    fn stage_variables(&self) -> StrMap;

    /// Configures instance with stage variables under #[cfg(test)] configurations
    ///
    /// This is intended for use in mock testing contexts.
    #[cfg(test)]
    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<StrMap>;

    /// Return request context data assocaited with the ALB or API gateway request
    fn request_context(&self) -> RequestContext;

    /// Return the Result of a payload parsed into a serde Deserializeable
    /// type
    ///
    /// Currently only `application/x-www-form-urlencoded`
    /// and `application/json` flavors of content type
    /// are supported
    ///
    /// A [PayloadError](enum.PayloadError.html) will be returned for undeserializable
    /// payloads. If no body is provided, `Ok(None)` will be returned.
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>;
}

impl RequestExt for http::Request<Body> {
    fn query_string_parameters(&self) -> StrMap {
        self.extensions()
            .get::<QueryStringParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or_default()
    }

    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<StrMap>,
    {
        let mut s = self;
        s.extensions_mut().insert(QueryStringParameters(parameters.into()));
        s
    }

    fn path_parameters(&self) -> StrMap {
        self.extensions()
            .get::<PathParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or_default()
    }

    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<StrMap>,
    {
        let mut s = self;
        s.extensions_mut().insert(PathParameters(parameters.into()));
        s
    }

    fn stage_variables(&self) -> StrMap {
        self.extensions()
            .get::<StageVariables>()
            .map(|ext| ext.0.clone())
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<StrMap>,
    {
        let mut s = self;
        s.extensions_mut().insert(StageVariables(variables.into()));
        s
    }

    fn request_context(&self) -> RequestContext {
        self.extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("Request did not contain a request context")
    }

    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>,
    {
        self.headers()
            .get(http::header::CONTENT_TYPE)
            .map(|ct| match ct.to_str() {
                Ok("application/x-www-form-urlencoded") => serde_urlencoded::from_bytes::<D>(self.body().as_ref())
                    .map_err(PayloadError::WwwFormUrlEncoded)
                    .map(Some),
                Ok("application/json") => serde_json::from_slice::<D>(self.body().as_ref())
                    .map_err(PayloadError::Json)
                    .map(Some),
                _ => Ok(None),
            })
            .unwrap_or_else(|| Ok(None))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Body, Request, RequestExt};
    use serde::Deserialize;

    #[test]
    fn requests_can_mock_query_string_parameters_ext() {
        let mocked = hashmap! {
            "foo".into() => vec!["bar".into()]
        };
        let request = Request::default().with_query_string_parameters(mocked.clone());
        assert_eq!(request.query_string_parameters(), mocked.into());
    }

    #[test]
    fn requests_can_mock_path_parameters_ext() {
        let mocked = hashmap! {
            "foo".into() => vec!["bar".into()]
        };
        let request = Request::default().with_path_parameters(mocked.clone());
        assert_eq!(request.path_parameters(), mocked.into());
    }

    #[test]
    fn requests_can_mock_stage_variables_ext() {
        let mocked = hashmap! {
            "foo".into() => vec!["bar".into()]
        };
        let request = Request::default().with_stage_variables(mocked.clone());
        assert_eq!(request.stage_variables(), mocked.into());
    }

    #[test]
    fn requests_have_form_post_parsable_payloads() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Payload {
            foo: String,
            baz: usize,
        }
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from("foo=bar&baz=2"))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_have_json_parseable_payloads() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Payload {
            foo: String,
            baz: usize,
        }
        let request = http::Request::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"foo":"bar", "baz": 2}"#))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_omiting_content_types_do_not_support_parseable_payloads() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Payload {
            foo: String,
            baz: usize,
        }
        let request = http::Request::builder()
            .body(Body::from(r#"{"foo":"bar", "baz": 2}"#))
            .expect("failed to bulid request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(payload, None);
    }
}
