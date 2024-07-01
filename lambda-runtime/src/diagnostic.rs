use serde::{Deserialize, Serialize};
use std::{any::type_name, borrow::Cow};

use crate::{deserializer::DeserializeError, Error};

/// Diagnostic information about an error.
///
/// `Diagnostic` is automatically derived for some common types,
/// like boxed types that implement [`Error`][std::error::Error].
/// If you use an error type which comes from a external crate like anyhow,
/// you need convert it to common types like `Box<dyn std::error::Error>`.
/// See the examples for more details.
///
/// [`error_type`][`Diagnostic::error_type`] is derived from the type name of
/// the original error with [`std::any::type_name`] as a fallback, which may
/// not be reliable for conditional error handling.
/// You can define your own error container that implements `Into<Diagnostic>`
/// if you need to handle errors based on error types.
///
/// Example:
/// ```
/// use lambda_runtime::{Diagnostic, Error, LambdaEvent};
/// use std::borrow::Cow;
///
/// #[derive(Debug)]
/// struct ErrorResponse(Error);
///
/// impl<'a> Into<Diagnostic<'a>> for ErrorResponse {
///     fn into(self) -> Diagnostic<'a> {
///         Diagnostic {
///             error_type: "MyError".into(),
///             error_message: self.0.to_string().into(),
///         }
///     }
/// }
///
/// async fn function_handler(_event: LambdaEvent<()>) -> Result<(), ErrorResponse> {
///    // ... do something
///    Ok(())
/// }
/// ```
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic<'a> {
    /// Error type.
    ///
    /// `error_type` is derived from the type name of the original error with
    /// [`std::any::type_name`] as a fallback.
    /// Please implement your own `Into<Diagnostic>` if you need more reliable
    /// error types.
    pub error_type: Cow<'a, str>,
    /// Error message.
    ///
    /// `error_message` is the output from the [`Display`][std::fmt::Display]
    /// implementation of the original error as a fallback.
    pub error_message: Cow<'a, str>,
}

impl<'a> From<DeserializeError> for Diagnostic<'a> {
    fn from(value: DeserializeError) -> Self {
        Diagnostic {
            error_type: type_name::<DeserializeError>().into(),
            error_message: value.to_string().into(),
        }
    }
}

impl<'a> From<Error> for Diagnostic<'a> {
    fn from(value: Error) -> Self {
        Diagnostic {
            error_type: type_name::<Error>().into(),
            error_message: value.to_string().into(),
        }
    }
}

impl<'a, T> From<Box<T>> for Diagnostic<'a>
where
    T: std::error::Error,
{
    fn from(value: Box<T>) -> Self {
        Diagnostic {
            error_type: type_name::<T>().into(),
            error_message: value.to_string().into(),
        }
    }
}

impl<'a> From<Box<dyn std::error::Error>> for Diagnostic<'a> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Diagnostic {
            error_type: type_name::<Box<dyn std::error::Error>>().into(),
            error_message: value.to_string().into(),
        }
    }
}

impl<'a> From<std::convert::Infallible> for Diagnostic<'a> {
    fn from(value: std::convert::Infallible) -> Self {
        Diagnostic {
            error_type: type_name::<std::convert::Infallible>().into(),
            error_message: value.to_string().into(),
        }
    }
}

impl<'a> From<String> for Diagnostic<'a> {
    fn from(value: String) -> Self {
        Diagnostic {
            error_type: type_name::<String>().into(),
            error_message: value.into(),
        }
    }
}

impl<'a> From<&'static str> for Diagnostic<'a> {
    fn from(value: &'static str) -> Self {
        Diagnostic {
            error_type: type_name::<&'static str>().into(),
            error_message: value.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn round_trip_lambda_error() {
        use serde_json::{json, Value};
        let expected = json!({
            "errorType": "InvalidEventDataError",
            "errorMessage": "Error parsing event data.",
        });

        let actual = Diagnostic {
            error_type: "InvalidEventDataError".into(),
            error_message: "Error parsing event data.".into(),
        };
        let actual: Value = serde_json::to_value(actual).expect("failed to serialize diagnostic");
        assert_eq!(expected, actual);
    }
}
