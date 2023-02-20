//! Extension methods for `Request` types

use std::{error::Error, fmt};

use serde::{
    de::{value::Error as SerdeError, DeserializeOwned},
    Deserialize,
};

use crate::Body;

/// Request payload deserialization errors
///
/// Returned by [`RequestPayloadExt::payload()`]
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
            PayloadError::Json(json) => writeln!(f, "failed to parse payload from application/json {json}"),
            PayloadError::WwwFormUrlEncoded(form) => writeln!(
                f,
                "failed to parse payload from application/x-www-form-urlencoded {form}"
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

/// Extensions for `lambda_http::Request` structs.
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
/// use lambda_http::{
///     service_fn, Body, Context, Error, IntoResponse, Request, RequestPayloadExt, Response,
/// };
/// use serde::Deserialize;
///
/// #[derive(Debug, Default, Deserialize)]
/// struct Args {
///   #[serde(default)]
///   x: usize,
///   #[serde(default)]
///   y: usize
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///   lambda_http::run(service_fn(add)).await?;
///   Ok(())
/// }
///
/// async fn add(
///   request: Request
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
pub trait RequestPayloadExt {
    /// Return the result of a payload parsed into a type that implements [`serde::Deserialize`]
    ///
    /// Currently only `application/x-www-form-urlencoded`
    /// and `application/json` flavors of content type
    /// are supported
    ///
    /// A [`PayloadError`] will be returned for undeserializable
    /// payloads. If no body is provided, `Ok(None)` will be returned.
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        D: DeserializeOwned;
}

impl RequestPayloadExt for http::Request<Body> {
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>,
    {
        self.headers()
            .get(http::header::CONTENT_TYPE)
            .map(|ct| match ct.to_str() {
                Ok(content_type) => {
                    if content_type.starts_with("application/x-www-form-urlencoded") {
                        return serde_urlencoded::from_bytes::<D>(self.body().as_ref())
                            .map_err(PayloadError::WwwFormUrlEncoded)
                            .map(Some);
                    } else if content_type.starts_with("application/json") {
                        return serde_json::from_slice::<D>(self.body().as_ref())
                            .map_err(PayloadError::Json)
                            .map(Some);
                    }

                    Ok(None)
                }
                _ => Ok(None),
            })
            .unwrap_or_else(|| Ok(None))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::RequestPayloadExt;

    use crate::Body;

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    struct Payload {
        foo: String,
        baz: usize,
    }

    #[test]
    fn requests_have_form_post_parsable_payloads() {
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
    fn requests_have_json_parsable_payloads() {
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
    fn requests_match_form_post_content_type_with_charset() {
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
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
    fn requests_match_json_content_type_with_charset() {
        let request = http::Request::builder()
            .header("Content-Type", "application/json; charset=UTF-8")
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
    fn requests_omitting_content_types_do_not_support_parsable_payloads() {
        let request = http::Request::builder()
            .body(Body::from(r#"{"foo":"bar", "baz": 2}"#))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(payload, None);
    }
}
