#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches the `lambda_runtime` crate with [http](https://github.com/hyperium/http)
//! types targeting ALB and API Gateway proxy events.
//!
//! Though ALB and API Gateway proxy events are separate Lambda triggers, they both share
//! similar shapes that contextually map to an http request handler. From a application perspective
//! the differences shouldn't matter. This crate
//! abstracts over both using standard [http](https://github.com/hyperium/http) types allowing
//! you to focus more on your application while giving you to the flexibility to
//! transparently use whichever http trigger suits your application's needs best.
//!
//! # Examples
//!
//! ```rust,no_run
//! use lambda_http::{lambda, IntoResponse, Request, RequestExt};
//! use lambda_runtime::{Context, error::HandlerError};
//!
//! fn main() {
//!     lambda!(hello)
//! }
//!
//! fn hello(
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

// only externed because maplit doesn't seem to play well with 2018 edition imports
#[cfg(test)]
#[macro_use]
extern crate maplit;

pub use http::{self, Response};
use lambda_runtime::{self as lambda, error::HandlerError, Context};
// use tokio::runtime::Runtime as TokioRuntime;
use futures::future::Future;

mod body;
mod ext;
pub mod request;
mod response;
mod strmap;

pub use crate::{body::Body, ext::RequestExt, response::IntoResponse, strmap::StrMap};
use crate::{request::LambdaRequest, response::LambdaResponse};

/// Type alias for `http::Request`s with a fixed `lambda_http::Body` body
pub type Request = http::Request<Body>;

/// Functions serving as ALB and API Gateway handlers must conform to this type.
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

/// Creates a new `lambda_runtime::Runtime` and begins polling for ALB and API Gateway events
///
/// # Arguments
///
/// * `f` A type that conforms to the `Handler` interface.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start<R>(f: impl Handler<R>/*, runtime: Option<TokioRuntime>*/) -> impl Future<Item=(), Error=()>
where
    R: IntoResponse,
{
    // handler requires a mutable ref
    let mut func = f;
    lambda::start(
        move |req: LambdaRequest<'_>, ctx: Context| {
            let is_alb = req.request_context.is_alb();
            func.run(req.into(), ctx)
                .map(|resp| LambdaResponse::from_response(is_alb, resp.into_response()))
        },
        // runtime,
    )
}

/// A macro for starting new handler's poll for API Gateway and ALB events
#[macro_export]
macro_rules! lambda {
    ($handler:expr) => {
        use tokio;
        tokio::run($crate::start($handler))
    };
    ($handler:expr, $runtime:expr) => {
        $runtime.spawn($crate::start($handler))
    };
    ($handler:ident) => {
        use tokio;
        tokio::run($crate::start($handler))
    };
    ($handler:ident, $runtime:expr) => {
        $runtime.spawn($crate::start($handler))
    };
}
