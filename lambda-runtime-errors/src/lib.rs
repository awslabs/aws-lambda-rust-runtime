//! The Lambda runtime errors crate defines the `LambdaErrorExt` trait
//! that can be used by libriaries to return errors compatible with the
//! AWS Lambda Rust runtime.
mod error_ext_impl;

pub use crate::error_ext_impl::*;

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
