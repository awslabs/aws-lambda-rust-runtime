#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches `lambda_runtime` with `http` types targeting API Gateway proxy events
//!
//! # Example
//!
//! ```rust,no_run
//! use lambda_http::{lambda, IntoResponse, Request, RequestExt};
//! use lambda_runtime::{Context, error::HandlerError};
//!
//! fn main() {
//!     lambda!(handler)
//! }
//!
//! fn handler(
//!     request: Request,
//!     _ctx: Context
//! ) -> Result<impl IntoResponse, HandlerError> {
//!     Ok(format!(
//!         "hello {}",
//!         request
//!             .query_string_parameters()
//!             .get("name")
//!             .unwrap_or_else(|| "stranger")
//!     ))
//! }
//! ```
//!
//! You can also provide a closure directly to the `lambda!` macro
//!
//! ```rust,no_run
//! use lambda_http::{lambda, Request, RequestExt};
//!
//! fn main() {
//!   lambda!(
//!     |request: Request, context| Ok(
//!       format!(
//!         "hello {}",
//!         request.query_string_parameters()
//!           .get("name")
//!           .unwrap_or_else(|| "stranger")
//!       )
//!     )
//!   )
//! }
//! ```

pub use http::{self, Response};
use lambda_runtime::{self as lambda, error::HandlerError, Context};
use tokio::runtime::Runtime as TokioRuntime;

mod body;
mod ext;
pub mod request;
mod response;
mod strmap;

pub use crate::{body::Body, ext::RequestExt, response::IntoResponse, strmap::StrMap};
use crate::{request::GatewayRequest, response::GatewayResponse};

/// Type alias for `http::Request`s with a fixed `lambda_http::Body` body
pub type Request = http::Request<Body>;

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
        |req: GatewayRequest<'_>, ctx: Context| {
            func.run(req.into(), ctx)
                .map(|resp| GatewayResponse::from(resp.into_response()))
        },
        runtime,
    )
}

/// A macro for starting new handler's poll for API Gateway events
#[macro_export]
macro_rules! lambda {
    ($handler:expr) => {
        $crate::start($handler, None)
    };
    ($handler:expr, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
    ($handler:ident) => {
        $crate::start($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
}
