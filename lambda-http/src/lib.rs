#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches the `lambda` crate with [http](https://github.com/hyperium/http)
//! types targeting ALB, API Gateway REST and HTTTP API proxy events.
//!
//! Though ALB and API Gateway proxy events are separate Lambda triggers, they both share
//! similar shapes that can be generalized to http request handler. From a application perspective
//! the differences shouldn't matter. This crate
//! abstracts over both using standard [http](https://github.com/hyperium/http) types allowing
//! you to focus more on your application while giving you to the flexibility to
//! transparently use whichever http trigger suits your application's needs best.
//!
//! # Examples
//!
//! ```rust,no_run
//! use lambda_http::{handler, Handler, lambda, IntoResponse, Request, RequestExt};
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     lambda::run(handler(hello).to_adapter()).await?;
//!     Ok(())
//! }
//!
//! async fn hello(
//!     request: Request,
//! ) -> Result<impl IntoResponse, Error> {
//!     Ok(format!(
//!         "hello {}",
//!         request
//!             .query_string_parameters()
//!             .get("name")
//!             .unwrap_or_else(|| "stranger")
//!     ))
//! }
//! ```

// only externed because maplit doesn't seem to play well with 2018 edition imports
#[cfg(test)]
#[macro_use]
extern crate maplit;

pub use http::{self, Response};
use lambda::Handler as LambdaHandler;
pub use lambda::{self};
mod body;
mod ext;
pub mod request;
mod response;
mod strmap;
pub use crate::{body::Body, ext::RequestExt, response::IntoResponse, strmap::StrMap};
use crate::{request::LambdaRequest, response::LambdaResponse};
use std::{
    error::Error,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type Err = Box<dyn Error + Send + Sync + 'static>;

/// Type alias for `http::Request`s with a fixed `lambda_http::Body` body
pub type Request = http::Request<Body>;

/// Functions serving as ALB and API Gateway REST and HTTP API handlers must conform to this type.
///
/// This can be viewed as a `lambda::Handler` constained to `http` crate `Request` and `Response` types
pub trait Handler: Sized {
    /// The type of Error that this Handler will return
    type Err;
    /// The type of Response this Handler will return
    type Response: IntoResponse;
    /// The type of Future this Handler will return
    type Fut: Future<Output = Result<Self::Response, Self::Err>> + 'static;
    /// Function used to execute handler behavior
    fn call(&mut self, event: Request) -> Self::Fut;

    /// Consumes this Handler into an Adapter type which implements `lambda::Hander`
    fn to_adapter(self) -> Adapter<Self> {
        Adapter { h: self }
    }
}

/// Coerse a type that implements `Handler` type into a `Handler`
pub fn handler<H: Handler>(h: H) -> H {
    h
}

/// An implementation of `Handler` for a given closure
impl<F, R, Fut> Handler for F
where
    F: FnMut(Request) -> Fut,
    R: IntoResponse,
    Fut: Future<Output = Result<R, Err>> + Send + 'static,
    Err: Into<Box<dyn Error + Send + Sync + 'static>> + fmt::Debug,
{
    type Response = R;
    type Err = Err;
    type Fut = Fut;
    fn call(&mut self, event: Request) -> Self::Fut {
        (*self)(event)
    }
}

#[doc(hidden)]
pub struct TransformResponse<R, E> {
    is_alb: bool,
    fut: Pin<Box<dyn Future<Output = Result<R, E>>>>,
}

impl<R, E> Future for TransformResponse<R, E>
where
    R: IntoResponse,
{
    type Output = Result<LambdaResponse, E>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.fut.as_mut().poll(cx) {
            Poll::Ready(result) => {
                Poll::Ready(result.map(|resp| LambdaResponse::from_response(self.is_alb, resp.into_response())))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Exists only to satisfy the trait cover rule for `lambda::Handler` impl for
//
/// See [this article](http://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/)
/// for a larger explaination of why this is nessessary
pub struct Adapter<H: Handler> {
    h: H,
}

impl<H: Handler> LambdaHandler<LambdaRequest<'_>, LambdaResponse> for Adapter<H> {
    type Err = H::Err;
    type Fut = TransformResponse<H::Response, Self::Err>;
    fn call(&mut self, event: LambdaRequest<'_>) -> Self::Fut {
        let is_alb = event.request_context.is_alb();
        let fut = Box::pin(self.h.call(event.into()));
        TransformResponse::<H::Response, Self::Err> { is_alb, fut }
    }
}
