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
//! use lambda_http::{Request, Response, RequestExt};
//!
//! fn main() {
//!   lambda!(handler)
//! }
//!
//! fn handler(
//!   request: Request,
//!   ctx: Context
//! ) -> Result<Response, HandlerError> {
//!   Ok(
//!     Response::new(
//!       format!(
//!         "hello {}",
//!         request.query_string_parameters()
//!           .get("name")
//!           .unwrap_or_else(|| "stranger")
//!       ).into()
//!     )
//!   )
//! }
//! ```
extern crate base64;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate http;
extern crate lambda_runtime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio;

use lambda_runtime::{self as lambda, Context};
use lambda_runtime::error::HandlerError;
use http::{Request as HttpRequest, Response as HttpResponse};
use tokio::runtime::Runtime as TokioRuntime;

mod ext;
pub mod request;
mod response;
mod body;
mod strmap;

use request::GatewayRequest;
use response::GatewayResponse;
pub use ext::RequestExt;
pub use strmap::StrMap;
pub use body::Body;

/// Type alias for `http::Request`s with a fixed `lambda_http::Body` body
pub type Request = HttpRequest<Body>;

/// Type alias for `http::Response`s with a fixed `lambda_http::Body` body
pub type Response = HttpResponse<Body>;

/// Functions acting as API Gateway handlers must conform to this type.
pub type Handler = fn(Request, Context) -> Result<Response, HandlerError>;


/// Creates a new `lambda_runtime::Runtime` and begins polling for API Gateway events
///
/// # Arguments
///
/// * `f` A type that conforms to the `Handler` interface.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start(f: Handler, runtime: Option<TokioRuntime>) {
    lambda::start(|req: GatewayRequest, ctx: Context| {
        f(req.into(), ctx).map(GatewayResponse::from)
    }, runtime)
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