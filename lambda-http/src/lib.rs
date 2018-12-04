#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches `lambda_runtime` with `http` types targeting API Gateway proxy events
//!
//! # Example
//!
//! ```rust,no_run
//! #[macro_use] extern crate lambda_http;
//! extern crate lambda_runtime as lambda;
//!
//! use lambda::{Context, HandlerError};
//! use lambda_http::{Request, IntoResponse, RequestExt};
//!
//! fn main() {
//!   lambda!(handler)
//! }
//!
//! fn handler(
//!   request: Request,
//!   ctx: Context
//! ) -> Result<impl IntoResponse, HandlerError> {
//!   Ok(
//!       format!(
//!         "hello {}",
//!         request.query_string_parameters()
//!           .get("name")
//!           .unwrap_or_else(|| "stranger")
//!       )
//!   )
//! }
//! ```
extern crate base64;
extern crate failure;
#[macro_use]
extern crate failure_derive;
/// re-export for convenient access in consumer crates
pub extern crate http;
extern crate lambda_runtime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio;

use http::Request as HttpRequest;
pub use http::Response;
use lambda_runtime::{self as lambda, error::HandlerError, Context};
use tokio::runtime::Runtime as TokioRuntime;

mod body;
mod ext;
pub mod request;
mod response;
mod strmap;

pub use body::Body;
pub use ext::RequestExt;
use request::GatewayRequest;
use response::GatewayResponse;
pub use response::IntoResponse;
pub use strmap::StrMap;

/// Type alias for `http::Request`s with a fixed `lambda_http::Body` body
pub type Request = HttpRequest<Body>;

/// Functions acting as API Gateway handlers must conform to this type.
pub trait Handler<R> {
    /// Run the handler.
    fn run(&mut self, event: Request, ctx: Context) -> Result<R, HandlerError>;
}

impl<F, R> Handler<R> for F
where
    F: FnMut(Request, Context) -> Result<R, HandlerError>,
{
    fn run(&mut self, event: Request, ctx: Context) -> Result<R, HandlerError> {
        (*self)(event, ctx)
    }
}

/// Creates a new `lambda_runtime::Runtime` and begins polling for API Gateway events
///
/// # Arguments
///
/// * `f` A type that conforms to the `Handler` interface.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start<R>(f: impl Handler<R>, runtime: Option<TokioRuntime>)
where
    R: IntoResponse,
{
    // handler requires a mutable ref
    let mut func = f;
    lambda::start(
        |req: GatewayRequest, ctx: Context| {
            func.run(req.into(), ctx)
                .map(|resp| GatewayResponse::from(resp.into_response()))
        },
        runtime,
    )
}

/// A macro for starting new handler's poll for API Gateway events
#[macro_export]
macro_rules! lambda {
    ($handler:ident) => {
        $crate::start($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
}
