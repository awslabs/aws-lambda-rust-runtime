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
//! The following example is how you would structure your Lambda such that you have a `main` function where you explicitly invoke
//! `lambda_runtime::run` in combination with the [`handler`](fn.handler.html) function. This pattern allows you to utilize global initialization
//! of tools such as loggers, to use on warm invokes to the same Lambda function after the first request, helping to reduce the latency of
//! your function's execution path.
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda_runtime::{self, Error}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // initialize dependencies once here for the lifetime of your
//!     // lambda task
//!     lambda_runtime::run(handler(|request, context| async { Ok("👋 world!") })).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Leveraging trigger provided data
//!
//! You can also access information provided directly from the underlying trigger events, like query string parameters,
//! with the [`RequestExt`](trait.RequestExt.html) trait.
//!
//! ```rust,no_run
//! use lambda_http::{handler, lambda_runtime::{self, Context, Error}, IntoResponse, Request, RequestExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     lambda_runtime::run(handler(hello)).await?;
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
pub use lambda_runtime::{self, Context};
use lambda_runtime::{Error, Handler as LambdaHandler};

mod body;
pub mod ext;
pub mod request;
mod response;
mod strmap;
pub use crate::{body::Body, ext::RequestExt, response::IntoResponse, strmap::StrMap};
use crate::{
    request::{LambdaRequest, RequestOrigin},
    response::LambdaResponse,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

/// Type alias for `http::Request`s with a fixed [`Body`](enum.Body.html) type
pub type Request = http::Request<Body>;

/// Functions serving as ALB and API Gateway REST and HTTP API handlers must conform to this type.
///
/// This can be viewed as a `lambda_runtime::Handler` constrained to `http` crate `Request` and `Response` types
pub trait Handler<'a>: Sized {
    /// The type of Error that this Handler will return
    type Error;
    /// The type of Response this Handler will return
    type Response: IntoResponse;
    /// The type of Future this Handler will return
    type Fut: Future<Output = Result<Self::Response, Self::Error>> + Send + 'a;
    /// Function used to execute handler behavior
    fn call(&self, event: Request, context: Context) -> Self::Fut;
}

/// Adapts a [`Handler`](trait.Handler.html) to the `lambda_runtime::run` interface
pub fn handler<'a, H: Handler<'a>>(handler: H) -> Adapter<'a, H> {
    Adapter {
        handler,
        _pd: PhantomData,
    }
}

/// An implementation of `Handler` for a given closure return a `Future` representing the computed response
impl<'a, F, R, Fut> Handler<'a> for F
where
    F: Fn(Request, Context) -> Fut,
    R: IntoResponse,
    Fut: Future<Output = Result<R, Error>> + Send + 'a,
{
    type Response = R;
    type Error = Error;
    type Fut = Fut;
    fn call(&self, event: Request, context: Context) -> Self::Fut {
        (self)(event, context)
    }
}

#[doc(hidden)]
pub struct TransformResponse<'a, R, E> {
    request_origin: RequestOrigin,
    fut: Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'a>>,
}

impl<'a, R, E> Future for TransformResponse<'a, R, E>
where
    R: IntoResponse,
{
    type Output = Result<LambdaResponse, E>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext) -> Poll<Self::Output> {
        match self.fut.as_mut().poll(cx) {
            Poll::Ready(result) => Poll::Ready(
                result.map(|resp| LambdaResponse::from_response(&self.request_origin, resp.into_response())),
            ),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Exists only to satisfy the trait cover rule for `lambda_runtime::Handler` impl
///
/// User code should never need to interact with this type directly. Since `Adapter` implements `Handler`
/// It serves as a opaque trait covering type.
///
/// See [this article](http://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/)
/// for a larger explaination of why this is nessessary
pub struct Adapter<'a, H: Handler<'a>> {
    handler: H,
    _pd: PhantomData<&'a H>,
}

impl<'a, H: Handler<'a>> Handler<'a> for Adapter<'a, H> {
    type Response = H::Response;
    type Error = H::Error;
    type Fut = H::Fut;
    fn call(&self, event: Request, context: Context) -> Self::Fut {
        self.handler.call(event, context)
    }
}

impl<'a, H: Handler<'a>> LambdaHandler<LambdaRequest<'a>, LambdaResponse> for Adapter<'a, H> {
    type Error = H::Error;
    type Fut = TransformResponse<'a, H::Response, Self::Error>;

    fn call(&self, event: LambdaRequest<'_>, context: Context) -> Self::Fut {
        let request_origin = event.request_origin();
        let fut = Box::pin(self.handler.call(event.into(), context));
        TransformResponse { request_origin, fut }
    }
}
