//! The error module defines the error types that can be returned
//! by custom handlers as well as the runtime itself.
use std::{env, error::Error, fmt};

use failure::Fail;

use backtrace;
use lambda_runtime_client::error;
use serde_json;

/// The `RuntimeError` object is returned by the custom runtime as it polls
/// for new events and tries to execute the handler function. The error
/// is primarily used by other methods within this crate and should not be relevant
/// to developers building Lambda functions. Handlers are expected to return
/// the `HandlerError` defined in this module.
#[derive(Debug)]
pub struct RuntimeError {
    msg: Box<dyn Fail>,
    stack_trace: Option<failure::Backtrace>,
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
    pub(crate) fn unrecoverable(msg: Box<dyn Fail>) -> RuntimeError {
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
    pub(crate) fn new(msg: Box<dyn Fail>) -> RuntimeError {
        let mut trace: Option<failure::Backtrace> = None;
        let is_backtrace = env::var("RUST_BACKTRACE");
        if is_backtrace.is_ok() && is_backtrace.unwrap() == "1" {
            trace!("Begin backtrace collection");
            trace = Option::from(failure::Backtrace::new());
            trace!("Completed backtrace collection");
        }
        RuntimeError {
            msg: Box::new(msg),
            stack_trace: trace,
            recoverable: true,
        }
    }
}

impl error::RuntimeApiError for RuntimeError {
    fn to_response(&self) -> error::ErrorResponse {
        let backtrace = format!("{:?}", self.stack_trace);
        error::ErrorResponse {
            error_message: String::from(self.description()),
            error_type: String::from(error::ERROR_TYPE_HANDLED),
            stack_trace: Option::from(backtrace.lines().map(|s| s.to_string()).collect::<Vec<String>>()),
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
        "blah"
    }

    fn cause(&self) -> Option<&Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

// impl From<env::VarError> for RuntimeError {
//     fn from(e: env::VarError) -> Self {
//         RuntimeError::unrecoverable(e)
//     }
// }

// impl From<serde_json::Error> for RuntimeError {
//     fn from(e: serde_json::Error) -> Self {
//         RuntimeError::unrecoverable(e.description())
//     }
// }

impl From<error::ApiError> for RuntimeError {
    fn from(e: error::ApiError) -> Self {
        let mut err = RuntimeError::new(Box::new(e));
        err.recoverable = e.is_recoverable();
        err.stack_trace = e.backtrace().map(|b| *b);
        err
    }
}

/// The error type for functions that are used as the `Handler` type. New errors
/// should be instantiated using the `new_error()` method  of the `runtime::Context`
/// object passed to the handler function.
#[derive(Debug, Clone)]
pub struct HandlerError {
    msg: String,
    backtrace: Option<backtrace::Backtrace>,
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// This is important for other errors to wrap this one.
impl Error for HandlerError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl HandlerError {
    /// Creates a new handler error. This method is used by the `new_error()` method
    /// of the `runtime::Context` object.
    ///
    /// # Arguments
    ///
    /// * `msg` The error message for the new error
    /// * `trace` A `Backtrace` object to generate the stack trace for the error
    ///           response. This is provided by the `Context` object.
    pub(crate) fn new(msg: &str, trace: Option<backtrace::Backtrace>) -> HandlerError {
        HandlerError {
            msg: msg.to_string(),
            backtrace: trace,
        }
    }
}

impl error::RuntimeApiError for HandlerError {
    fn to_response(&self) -> error::ErrorResponse {
        let backtrace = format!("{:?}", self.backtrace);
        error::ErrorResponse {
            error_message: String::from(self.description()),
            error_type: String::from(error::ERROR_TYPE_HANDLED),
            stack_trace: Option::from(backtrace.lines().map(|s| s.to_string()).collect::<Vec<String>>()),
        }
    }
}
