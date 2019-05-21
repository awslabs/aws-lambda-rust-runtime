use crate::context::Context;
use failure::Fail;
use lambda_runtime_errors::LambdaErrorExt;
use std::fmt::Display;
use tokio::prelude::future::IntoFuture;

/// Functions acting as a handler must conform to this type.
pub trait Handler<EventError, Fut>: Send {
    /// Run the handler.
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Fut;
}

impl<'ev, Function, EventError, Fut> Handler<EventError, Fut> for Function
where
    Function: FnMut(Vec<u8>, Context) -> Fut + Send,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    Fut: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
{
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Fut {
        (*self)(event, ctx)
    }
}
