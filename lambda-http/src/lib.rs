#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches the `lambda` crate with [`http`](https://github.com/hyperium/http)
//! types targeting AWS [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html), [API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html) REST and HTTP API lambda integrations.
//!
//! This crate abstracts over all of these trigger events using standard [`http`](https://github.com/hyperium/http) types minimizing the mental overhead
//! of the implementation details allowing you to focus more on your application while also giving you to the maximum flexibility to
//! transparently use whichever lambda trigger suits your application's needs best.
//!
//! # Examples
//!
//! ## Hello world
//!
//! lambda_http handlers adapt to the standard `lambda::Handler` interface using the [`handler`](fn.handler.html) function.
//!
//! The simplest case http handler is a closure of `http::Request` to a type that can be lifted into an `http::Response`.
//! You can learn more about these types [here](trait.IntoResponse.html)
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda};
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     lambda::run(handler(|req| async { Ok("👋 world!") })).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Hello world (simpler)
//!
//! For the simple cases you may not need much if any bootstrapping. To make life simpler
//! you can add an `#[lambda_http]` attribute to your `main` function and `lambda::run` machinery
//! will be wired in for you.
//!
//! Note: this comes at the expense of any one time `main` function initialization for your lambda task might find value in.
//! The body of your `main` function will be executed on every invocation of your lambda task.
//!
//! ```rust,no_run
//! use lambda_http::{lambda_http, Request, IntoResponse};
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[lambda_http]
//! #[tokio::main]
//! async fn main(req: Request) -> Result<impl IntoResponse, Error> {
//!     Ok("👋 world!")
//! }
//! ```
//!
//! ## Leveraging trigger provided data
//!
//! You can also access information provided directly from the underlying trigger events, like query string parameters,
//! with the [`RequestExt`](trait.RequestExt.html) trait.
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda, IntoResponse, Request, RequestExt};
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     lambda::run(handler(hello)).await?;
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
pub use lambda_http_attributes::lambda_http;
mod body;
pub mod ext;
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

/// Type alias for `http::Request`s with a fixed [`Body`](enum.Body.html) type
pub type Request = http::Request<Body>;

/// Functions serving as ALB and API Gateway REST and HTTP API handlers must conform to this type.
///
/// This can be viewed as a `lambda::Handler` constrained to `http` crate `Request` and `Response` types
pub trait Handler: Sized {
    /// The type of Error that this Handler will return
    type Err;
    /// The type of Response this Handler will return
    type Response: IntoResponse;
    /// The type of Future this Handler will return
    type Fut: Future<Output = Result<Self::Response, Self::Err>> + 'static;
    /// Function used to execute handler behavior
    fn call(&mut self, event: Request) -> Self::Fut;
}

/// Adapts a [`Handler`](trait.Handler.html) to the `lambda::run` interface
pub fn handler<H: Handler>(handler: H) -> Adapter<H> {
    Adapter { handler }
}

/// An implementation of `Handler` for a given closure return a `Future` representing the computed response
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

/// Exists only to satisfy the trait cover rule for `lambda::Handler` impl
///
/// User code should never need to interact with this type directly. Since `Adapter` implements `Handler`
/// It serves as a opaque trait covering type.
///
/// See [this article](http://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/)
/// for a larger explaination of why this is nessessary
pub struct Adapter<H: Handler> {
    handler: H,
}

impl<H: Handler> Handler for Adapter<H> {
    type Response = H::Response;
    type Err = H::Err;
    type Fut = H::Fut;
    fn call(&mut self, event: Request) -> Self::Fut {
        self.handler.call(event)
    }
}

impl<H: Handler> LambdaHandler<LambdaRequest<'_>, LambdaResponse> for Adapter<H> {
    type Err = H::Err;
    type Fut = TransformResponse<H::Response, Self::Err>;
    fn call(&mut self, event: LambdaRequest<'_>) -> Self::Fut {
        let is_alb = event.is_alb();
        let fut = Box::pin(self.handler.call(event.into()));
        TransformResponse { is_alb, fut }
    }
}
