#![warn(missing_docs)]
#![deny(warnings)]
//! Lambda runtime makes it easy to run Rust code inside AWS Lambda. To create
//! Lambda function with this library simply include it as a dependency, make
//! sure that you declare a function that respects the `Handler` type, and call
//! the `start()` function from your main method. The executable in your deployment
//! package must be called `bootstrap`.
//!
//! ```rust,no_run
//! use lambda_runtime::{error::HandlerError, lambda, Context};
//! use simple_error::bail;
//! use serde_derive::{Serialize, Deserialize};
//!
//! #[derive(Deserialize, Clone)]
//! struct CustomEvent {
//!     first_name: String,
//!     last_name: String,
//! }
//!
//! #[derive(Serialize, Clone)]
//! struct CustomOutput {
//!     message: String,
//! }
//!
//! fn main() {
//!     lambda!(my_handler);
//! }
//!
//! fn my_handler(e: CustomEvent, ctx: Context) -> Result<CustomOutput, HandlerError> {
//!     if e.first_name == "" {
//!         bail!("Empty first name");
//!     }
//!     Ok(CustomOutput{
//!         message: format!("Hello, {}!", e.first_name),
//!     })
//! }
//! ```
use failure::Fail;
use lambda_runtime_core::{start_with_config, EnvConfigProvider, HandlerError, LambdaErrorExt};
use serde;
use serde_json;
use std::fmt::Display;
use tokio::runtime::Runtime as TokioRuntime;

pub use lambda_runtime_core::Context;

/// The error module exposes the HandlerError type.
pub mod error {
    pub use lambda_runtime_core::{HandlerError, LambdaErrorExt};
}

/// Functions acting as a handler must conform to this type.
pub trait Handler<Event, Output, EventError> {
    /// Method to execute the handler function
    fn run(&mut self, event: Event, ctx: Context) -> Result<Output, EventError>;
}

/// Implementation of the `Handler` trait for both function pointers
/// and closures.
impl<Function, Event, Output, EventError> Handler<Event, Output, EventError> for Function
where
    Function: FnMut(Event, Context) -> Result<Output, EventError>,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
{
    fn run(&mut self, event: Event, ctx: Context) -> Result<Output, EventError> {
        (*self)(event, ctx)
    }
}

/// Wraps a typed handler into a closure that complies with the `Handler` trait
/// defined in the `lambda_runtime_core` crate. The closure simply uses `serde_json`
/// to serialize and deserialize the incoming event from a `Vec<u8>` and the output
/// to a `Vec<u8>`.
fn wrap<Event, Output, EventError>(
    mut h: impl Handler<Event, Output, EventError>,
) -> impl FnMut(Vec<u8>, Context) -> Result<Vec<u8>, HandlerError>
where
    Event: serde::de::DeserializeOwned,
    Output: serde::Serialize,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
{
    move |ev, ctx| {
        let event: Event = serde_json::from_slice(&ev)?;
        match h.run(event, ctx) {
            Ok(out) => {
                let out_bytes = serde_json::to_vec(&out)?;
                Ok(out_bytes)
            }
            Err(e) => Err(HandlerError::new(e)),
        }
    }
}

/// Creates a new runtime and begins polling for events using Lambda's Runtime APIs.
///
/// # Arguments
///
/// * `f` A function pointer that conforms to the `Handler` type.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start<Event, Output, EventError>(f: impl Handler<Event, Output, EventError>, runtime: Option<TokioRuntime>)
where
    Event: serde::de::DeserializeOwned,
    Output: serde::Serialize,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
{
    let wrapped = wrap(f);
    start_with_config(wrapped, &EnvConfigProvider::default(), runtime)
}

#[macro_export]
macro_rules! lambda {
    ($handler:ident) => {
        $crate::start($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
    ($handler:expr) => {
        $crate::start($handler, None)
    };
    ($handler:expr, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use lambda_runtime_core::Context;
    use serde_derive::{Deserialize, Serialize};
    use serde_json;

    fn test_context() -> Context {
        Context {
            memory_limit_in_mb: 128,
            function_name: "test_func".to_string(),
            function_version: "$LATEST".to_string(),
            invoked_function_arn: "arn:aws:lambda".to_string(),
            aws_request_id: "123".to_string(),
            xray_trace_id: "123".to_string(),
            log_stream_name: "logStream".to_string(),
            log_group_name: "logGroup".to_string(),
            client_context: Option::default(),
            identity: Option::default(),
            deadline: 0,
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Input {
        name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Output {
        msg: String,
    }

    #[test]
    fn runtime_invokes_handler() {
        let handler_ok = |_e: Input, _c: Context| -> Result<Output, HandlerError> {
            Ok(Output {
                msg: "hello".to_owned(),
            })
        };
        let mut wrapped_ok = wrap(handler_ok);
        let input = Input {
            name: "test".to_owned(),
        };
        let output = wrapped_ok.run(
            serde_json::to_vec(&input).expect("Could not convert input to Vec"),
            test_context(),
        );
        assert_eq!(
            output.is_err(),
            false,
            "Handler threw an unexpected error: {}",
            output.err().unwrap()
        );
        let output_obj: Output = serde_json::from_slice(&output.ok().unwrap()).expect("Could not serialize output");
        assert_eq!(
            output_obj.msg,
            "hello".to_owned(),
            "Unexpected output message: {}",
            output_obj.msg
        );
    }

    #[test]
    fn runtime_captures_error() {
        let handler_ok = |e: Input, _c: Context| -> Result<Output, HandlerError> {
            let _age = e.name.parse::<u8>()?;
            Ok(Output {
                msg: "hello".to_owned(),
            })
        };
        let mut wrapped_ok = wrap(handler_ok);
        let input = Input {
            name: "test".to_owned(),
        };
        let output = wrapped_ok.run(
            serde_json::to_vec(&input).expect("Could not convert input to Vec"),
            test_context(),
        );
        assert_eq!(output.is_err(), true, "Handler did not throw an error");
        let err = output.err().unwrap();
        assert_eq!(
            err.error_type(),
            "std::num::ParseIntError",
            "Unexpected error_type: {}",
            err.error_type()
        );
    }
}
