#![warn(missing_docs)]
//#![deny(warnings)]
//! Enriches the `lambda` crate with [`http`](https://github.com/hyperium/http)
//! types targeting AWS [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html), [API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html) REST and HTTP API lambda integrations.
//!
//! This crate abstracts over all of these trigger events using standard [`http`](https://github.com/hyperium/http) types minimizing the mental overhead
//! of understanding the nuances and variation between trigger details allowing you to focus more on your application while also giving you to the maximum flexibility to
//! transparently use whichever lambda trigger suits your application and cost optimiztions best.
//!
//! # Examples
//!
//! ## Hello World
//!
//! `lambda_http` handlers adapt to the standard `lambda::Handler` interface using the [`handler`](fn.handler.html) function.
//!
//! The simplest case of an http handler is a function of an `http::Request` to a type that can be lifted into an `http::Response`.
//! You can learn more about these types [here](trait.IntoResponse.html).
//!
//! Adding an `#[lambda(http)]` attribute to a `#[tokio::run]`-decorated `main` function will setup and run the Lambda function.
//!
//! Note: this comes at the expense of any onetime initialization your lambda task might find value in.
//! The full body of your `main` function will be executed on **every** invocation of your lambda task.
//!
//! ```rust,no_run
//! use lambda_http::{lambda::{lambda, Context}, Request, IntoResponse};
//!
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[lambda(http)]
//! #[tokio::main]
//! async fn main(_: Request, _: Context) -> Result<impl IntoResponse, Error> {
//!     Ok("👋 world!")
//! }
//! ```
//!
//! ## Hello World, Without Macros
//!
//! For cases where your lambda might benfit from one time function initializiation might
//! prefer a plain `main` function and invoke `lambda::run` explicitly in combination with the [`handler`](fn.handler.html) function.
//! Depending on the runtime cost of your dependency bootstrapping, this can reduce the overall latency of your functions execution path.
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda};
//!
//! type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // initialize dependencies once here for the lifetime of your
//!     // lambda task
//!     lambda::run(handler(|request, context| async { Ok("👋 world!") })).await?;
//!     Ok(())
//! }
//!
//! ```
//!
//! ## Leveraging trigger provided data
//!
//! You can also access information provided directly from the underlying trigger events, like query string parameters,
//! with the [`RequestExt`](trait.RequestExt.html) trait.
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda::{self, Context}, IntoResponse, Request, RequestExt};
//!
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
//!     _: Context
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
pub use lambda::{self, Context};
pub use lambda_attributes::lambda;

mod body;
pub mod ext;
pub mod request;
mod response;
mod strmap;
pub use crate::{body::Body, ext::RequestExt, response::IntoResponse, strmap::StrMap};
use crate::{request::LambdaRequest, response::LambdaResponse};
use std::{
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

/// Error type that lambdas may result in
pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Type alias for `http::Request`s with a fixed [`Body`](enum.Body.html) type
pub type Request = http::Request<Body>;

/// Functions serving as ALB and API Gateway REST and HTTP API handlers must conform to this type.
///
/// This can be viewed as a `lambda::Handler` constrained to `http` crate `Request` and `Response` types
pub trait Handler: Sized {
    /// The type of Error that this Handler will return
    type Error;
    /// The type of Response this Handler will return
    type Response: IntoResponse;
    /// The type of Future this Handler will return
    type Fut: Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static;
    /// Function used to execute handler behavior
    fn call(&self, event: Request, context: Context) -> Self::Fut;
}

/// Adapts a [`Handler`](trait.Handler.html) to the `lambda::run` interface
pub fn handler<H: Handler>(handler: H) -> Adapter<H> {
    Adapter { handler }
}

/// An implementation of `Handler` for a given closure return a `Future` representing the computed response
impl<F, R, Fut> Handler for F
where
    F: Fn(Request, Context) -> Fut,
    R: IntoResponse,
    Fut: Future<Output = Result<R, Error>> + Send + Sync + 'static,
{
    type Response = R;
    type Error = Error;
    type Fut = Fut;
    fn call(&self, event: Request, context: Context) -> Self::Fut {
        (self)(event, context)
    }
}

#[doc(hidden)]
pub struct TransformResponse<R, E> {
    is_alb: bool,
    fut: Pin<Box<dyn Future<Output = Result<R, E>> + Send + Sync>>,
}

impl<R, E> Future for TransformResponse<R, E>
where
    R: IntoResponse,
{
    type Output = Result<LambdaResponse, E>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext) -> Poll<Self::Output> {
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
    type Error = H::Error;
    type Fut = H::Fut;
    fn call(&self, event: Request, context: Context) -> Self::Fut {
        self.handler.call(event, context)
    }
}

impl<H: Handler> LambdaHandler<LambdaRequest<'_>, LambdaResponse> for Adapter<H> {
    type Error = H::Error;
    type Fut = TransformResponse<H::Response, Self::Error>;
    fn call(&self, event: LambdaRequest<'_>, context: Context) -> Self::Fut {
        let is_alb = event.is_alb();
        let fut = Box::pin(self.handler.call(event.into(), context));
        TransformResponse { is_alb, fut }
    }
}
