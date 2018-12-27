#![warn(missing_docs)]
#![deny(warnings)]
//! Lambda runtime makes it easy to run Rust code inside AWS Lambda. To create
//! Lambda function with this library simply include it as a dependency, make
//! sure that you declare a function that respects the `Handler` type, and call
//! the `start()` function from your main method. The executable in your deployment
//! package must be called `bootstrap`.
//!
//! ```rust,no_run
//! extern crate serde_derive;
//! extern crate lambda_runtime;
//! extern crate simple_error;
//!
//! use lambda_runtime::{HandlerError, lambda};
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
//! fn my_handler(e: CustomEvent, ctx: lambda_runtime::Context) -> Result<CustomOutput, HandlerError> {
//!     if e.first_name == "" {
//!         bail!("Empty first name");
//!     }
//!     Ok(CustomOutput{
//!         message: format!("Hello, {}!", e.first_name),
//!     })
//! }
//! ```
use lambda_runtime_core::{start_with_config, EnvConfigProvider};
use serde;
use serde_json;
use tokio::runtime::Runtime as TokioRuntime;

pub use lambda_runtime_core::Context;

/// The error module exposes the HandlerError type.
pub mod error {
    pub use lambda_runtime_core::HandlerError;
}

/// Functions acting as a handler must conform to this type.
pub trait Handler<E, O> {
    /// Method to execute the handler function
    fn run(&mut self, event: E, ctx: Context) -> Result<O, error::HandlerError>;
}

/// Implementation of the `Handler` trait for both function pointers
/// and closures.
impl<F, E, O> Handler<E, O> for F
where
    F: FnMut(E, Context) -> Result<O, error::HandlerError>,
{
    fn run(&mut self, event: E, ctx: Context) -> Result<O, error::HandlerError> {
        (*self)(event, ctx)
    }
}

/// Wraps a typed handler into a closure that complies with the `Handler` trait
/// defined in the `lambda_runtime_core` crate. The closure simply uses `serde_json`
/// to serialize and deserialize the incoming event from a `Vec<u8>` and the output
/// to a `Vec<u8>`.
fn wrap<E, O>(mut h: impl Handler<E, O>) -> impl FnMut(Vec<u8>, Context) -> Result<Vec<u8>, error::HandlerError>
where
    E: serde::de::DeserializeOwned,
    O: serde::Serialize,
{
    move |ev, ctx| {
        let event: E = serde_json::from_slice(&ev)?;
        match h.run(event, ctx) {
            Ok(out) => {
                let out_bytes = serde_json::to_vec(&out)?;
                Ok(out_bytes)
            }
            Err(err) => Err(err),
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
pub fn start<E, O>(f: impl Handler<E, O>, runtime: Option<TokioRuntime>)
where
    E: serde::de::DeserializeOwned,
    O: serde::Serialize,
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
