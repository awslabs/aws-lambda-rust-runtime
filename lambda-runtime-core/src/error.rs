//! The error module defines the error types that can be returned
//! by custom handlers as well as the runtime itself.
use std::{env, error::Error, fmt};

use lambda_runtime_client::error::ApiError;
use lambda_runtime_errors::LambdaErrorExt;

/// The `RuntimeError` object is returned by the custom runtime as it polls
/// for new events and tries to execute the handler function. The error
/// is primarily used by other methods within this crate and should not be relevant
/// to developers building Lambda functions. Handlers are expected to return
/// the `HandlerError` defined in this module.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    msg: String,
    /// The request id that generated this error
    pub(crate) request_id: Option<String>,
    /// Whether the error is recoverable or not.
    pub(crate) recoverable: bool,
}

impl RuntimeError {
    /// Creates a new `RuntimeError` that is unrecoverable and it will cause the
    /// runtime to panic in order to force a restart of the execution environment.
    /// When a new `RuntimeError` is created the stack trace for the error is collected
    /// automatically using the `backtrace` crate.
    ///
    /// # Arguments
    ///
    /// * `msg` The error message to be attached to the error.
    ///
    /// # Returns
    /// A new `RuntimeError` instance with the `recoverable` property set to `false`.
    pub(crate) fn unrecoverable(msg: &str) -> RuntimeError {
        let mut new_error = RuntimeError::new(msg);
        new_error.recoverable = false;
        new_error
    }

    /// Creates a new `RuntimeError` with the given properties. The stack trace for the
    /// error is collected automatically using the `backtrace` crate.
    ///
    /// # Arguments
    ///
    /// * `msg` The error message
    ///
    /// # Returns
    /// A new `RuntimeError` instance.
    pub(crate) fn new(msg: &str) -> RuntimeError {
        RuntimeError {
            msg: String::from(msg),
            recoverable: true,
            request_id: None,
        }
    }
}

impl LambdaErrorExt for RuntimeError {
    fn error_type(&self) -> &str {
        if self.recoverable {
            "RecoverableRuntimeError"
        } else {
            "UnrecoverableRuntimeError"
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// This is important for other errors to wrap this one.
impl Error for RuntimeError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl From<env::VarError> for RuntimeError {
    fn from(e: env::VarError) -> Self {
        RuntimeError::unrecoverable(e.description())
    }
}

impl From<ApiError> for RuntimeError {
    fn from(e: ApiError) -> Self {
        let mut err = RuntimeError::new(&format!("{}", e));
        err.recoverable = e.is_recoverable();
        err
    }
}
