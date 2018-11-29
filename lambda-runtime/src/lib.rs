#![warn(missing_docs)]
#![deny(warnings)]
//! Lambda runtime makes it easy to run Rust code inside AWS Lambda. To create
//! Lambda function with this library simply include it as a dependency, make
//! sure that you declare a function that respects the `Handler` type, and call
//! the `start()` function from your main method. The executable in your deployment
//! package must be called `bootstrap`.
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate serde_derive;
//! #[macro_use]
//! extern crate lambda_runtime;
//!
//! use lambda_runtime::error::HandlerError;
//!
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
//!         return Err(ctx.new_error("Missing first name!"));
//!     }
//!     Ok(CustomOutput{
//!         message: format!("Hello, {}!", e.first_name),
//!     })
//! }
//! ```
#[macro_use]
extern crate log;

extern crate backtrace;
extern crate chrono;
extern crate lambda_runtime_client;
extern crate serde;
extern crate serde_json;
extern crate tokio;

mod context;
mod env;
pub mod error;
mod runtime;

pub use context::*;
pub use runtime::*;
