use serde::{Deserialize, Serialize};
use std::any::type_name;

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
///
/// #[derive(Debug)]
/// struct ErrorResponse(&'static str);
///
/// impl Into<Diagnostic> for ErrorResponse {
///     fn into(self) -> Diagnostic {
///         Diagnostic {
///             error_type: "MyError".into(),
///             error_message: self.0.to_string(),
///         }
///     }
/// }
///
/// async fn function_handler(_event: LambdaEvent<()>) -> Result<(), ErrorResponse> {
///    Err(ErrorResponse("this is an error response"))
/// }
/// ```
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// `error_type` is the type of exception or error returned by the function.
    /// Use this field to categorize the different kinds of errors that your function
    /// might experience.
    ///
    /// In standard implementations, `error_type` is derived from the type name of the original error with
    /// [`std::any::type_name`], however this is not descriptive enough for an error type.
    /// Implement your own `Into<Diagnostic>` to return a more descriptive error type.
    pub error_type: String,
    /// `error_message` is a string expression of the error.
    /// In standard implementations, it's the output from the [`Display`][std::fmt::Display]
    /// implementation of the original error.
    pub error_message: String,
}

/// Trait that adds an `into_diagnostic` method to several types,
/// including standard errors, and error types from several well known error crates.
///
/// This trait is automatically implemented for some common types,
/// like boxed types that implement [`Error`][std::error::Error].
/// If you use an error type which comes from a external crate like anyhow, eyre, or miette,
/// you can enable the features `anyhow`, `eyre` or `miette` in the `lambda_runtime` crate to automatically
/// implement `IntoDiagnostic`.
///
/// Example:
/// ```
/// use lambda_runtime::{Diagnostic, IntoDiagnostic, LambdaEvent};
///
/// #[derive(Debug)]
/// struct ErrorResponse(&'static str);
///
/// impl IntoDiagnostic for ErrorResponse {
///     fn into_diagnostic(self) -> Diagnostic {
///         Diagnostic {
///             error_type: "MyError".into(),
///             error_message: self.0.into(),
///         }
///     }
/// }
///
/// async fn function_handler(_event: LambdaEvent<()>) -> Result<(), Diagnostic> {
///    Err(ErrorResponse("this is an error response").into_diagnostic())
/// }
/// ```
pub trait IntoDiagnostic {
    /// Converts external error types into [`Diagnostic`]
    fn into_diagnostic(self) -> Diagnostic;
}

impl IntoDiagnostic for DeserializeError {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl IntoDiagnostic for Error {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl IntoDiagnostic for Box<dyn std::error::Error> {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl IntoDiagnostic for std::convert::Infallible {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl IntoDiagnostic for String {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self,
        }
    }
}

impl IntoDiagnostic for &'static str {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.into(),
        }
    }
}

impl IntoDiagnostic for std::io::Error {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl<T> IntoDiagnostic for Box<T>
where
    T: std::error::Error,
{
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

impl From<DeserializeError> for Diagnostic {
    fn from(value: DeserializeError) -> Self {
        value.into_diagnostic()
    }
}

impl From<Error> for Diagnostic {
    fn from(value: Error) -> Self {
        value.into_diagnostic()
    }
}

impl<T> From<Box<T>> for Diagnostic
where
    T: std::error::Error,
{
    fn from(value: Box<T>) -> Self {
        value.into_diagnostic()
    }
}

impl From<Box<dyn std::error::Error>> for Diagnostic {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        value.into_diagnostic()
    }
}

impl From<std::convert::Infallible> for Diagnostic {
    fn from(value: std::convert::Infallible) -> Self {
        value.into_diagnostic()
    }
}

impl From<String> for Diagnostic {
    fn from(value: String) -> Self {
        value.into_diagnostic()
    }
}

impl From<&'static str> for Diagnostic {
    fn from(value: &'static str) -> Self {
        value.into_diagnostic()
    }
}

impl From<std::io::Error> for Diagnostic {
    fn from(value: std::io::Error) -> Self {
        value.into_diagnostic()
    }
}

#[cfg(feature = "anyhow")]
impl IntoDiagnostic for anyhow::Error {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

#[cfg(feature = "eyre")]
impl IntoDiagnostic for eyre::Report {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

#[cfg(feature = "miette")]
impl IntoDiagnostic for miette::Report {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            error_type: type_name_of_val(&self),
            error_message: self.to_string(),
        }
    }
}

pub(crate) fn type_name_of_val<T>(_: T) -> String {
    type_name::<T>().into()
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

    #[cfg(feature = "anyhow")]
    #[test]
    fn test_anyhow_integration() {
        use anyhow::Error as AnyhowError;
        let error: AnyhowError = anyhow::anyhow!("anyhow error");
        let diagnostic: Diagnostic = error.into_diagnostic();
        assert_eq!(diagnostic.error_type, "&anyhow::Error");
        assert_eq!(diagnostic.error_message, "anyhow error");
    }

    #[cfg(feature = "eyre")]
    #[test]
    fn test_eyre_integration() {
        use eyre::Report;
        let error: Report = eyre::eyre!("eyre error");
        let diagnostic: Diagnostic = error.into_diagnostic();
        assert_eq!(diagnostic.error_type, "&eyre::Report");
        assert_eq!(diagnostic.error_message, "eyre error");
    }

    #[cfg(feature = "miette")]
    #[test]
    fn test_miette_integration() {
        use miette::Report;
        let error: Report = miette::miette!("miette error");
        let diagnostic: Diagnostic = error.into_diagnostic();
        assert_eq!(diagnostic.error_type, "&miette::eyreish::Report");
        assert_eq!(diagnostic.error_message, "miette error");
    }
}
