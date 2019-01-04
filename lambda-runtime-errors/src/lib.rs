//! The Lambda runtime errors crate defines the `LambdaErrorExt` trait
//! that can be used by libriaries to return errors compatible with the
//! AWS Lambda Rust runtime.
mod error_ext_impl;

pub use crate::error_ext_impl::*;

use failure::{format_err, Compat, Error};
use std::fmt;

/// The `LambdaErrorExt` trait defines the `error_type()` method used
/// by the AWS Lambda runtime client to generate `ErrorResponse`
/// objects. The value returned by the `error_type()` method is used to
/// populate the `errorType` field in the Lambda response.  This crate
/// includes an implementation of this trait for most errors in the
/// standard library. By default, error return their type name.
pub trait LambdaErrorExt {
    /// The value for this field should be an alphanumeric unique identifier
    /// of the error type. For example `MyCustomError`.
    ///
    /// # Return
    /// An alphanumeric identifier for the error
    fn error_type(&self) -> &str;
}

impl LambdaErrorExt for Error {
    fn error_type(&self) -> &str {
        if let Some(name) = self.find_root_cause().name() {
            return name;
        }
        "FailureError"
    }
}

// We implement this trait here so that we can use the Compat type
// in the lambda-runtime crate - heaps of fun between failure and std::error
impl LambdaErrorExt for Compat<Error> {
    fn error_type(&self) -> &str {
        "CompatFailureError"
    }
}

/// The `HandlerError` struct can be use to abstract any `Err` of the handler method `Result`.
/// The `HandlerError` object can be generated `From` any object that supports `Display`,
/// `Send, `Sync`, and `Debug`. This allows handler functions to return any error using
/// the `?` syntax. For example `let _age_num: u8 = e.age.parse()?;` will return the
/// `<F as FromStr>::Err` from the handler function.
//pub type HandlerError = failure::Error;
#[derive(Debug)]
pub struct HandlerError {
    err_type: String,
    inner: failure::Error,
}
impl HandlerError {
    pub fn new<T: failure::Fail + LambdaErrorExt + Send + Sync>(e: T) -> Self {
        let err_type = e.error_type().to_owned();
        HandlerError {
            err_type,
            inner: failure::Error::from(e),
        }
    }
}
impl std::error::Error for HandlerError {}
impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.err_type, self.inner.find_root_cause())
    }
}
impl LambdaErrorExt for HandlerError {
    fn error_type(&self) -> &str {
        &self.err_type
    }
}
impl From<&str> for HandlerError {
    fn from(s: &str) -> Self {
        HandlerError {
            err_type: "UnknownError".to_owned(),
            inner: format_err!("{}", s),
        }
    }
}
impl From<failure::Error> for HandlerError {
    fn from(e: failure::Error) -> Self {
        let error_type = e.error_type();
        HandlerError {
            err_type: error_type.to_owned(),
            inner: e,
        }
    }
}
impl From<serde_json::error::Error> for HandlerError {
    fn from(e: serde_json::error::Error) -> Self {
        HandlerError {
            err_type: "JsonError".to_owned(),
            inner: failure::Error::from(e),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use failure::Fail;

    #[derive(Fail, Debug)]
    #[fail(display = "Custom Error")]
    struct CustomError;

    #[test]
    fn std_error_type() {
        let parsed_int = "hello".parse::<u8>();
        let err = HandlerError::from(parsed_int.err().unwrap());
        assert_eq!(err.error_type(), "std::num::ParseIntError");
    }

    #[test]
    fn error_type_from_failure() {
        let err = HandlerError::from(failure::Error::from(CustomError {}));
        assert_eq!(err.error_type(), "lambda_runtime_errors::tests::CustomError");
    }
}
