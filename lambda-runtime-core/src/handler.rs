use crate::context::Context;
use failure::Fail;
use lambda_runtime_errors::LambdaErrorExt;
use std::fmt::Display;
use tokio::prelude::future::IntoFuture;

/// Functions acting as a handler must conform to this type.
pub trait Handler<EventError, I>: Send {
    /// Run the handler.
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> I;
}

impl<'ev, Function, EventError, I> Handler<EventError, I> for Function
where
    Function: FnMut(Vec<u8>, Context) -> I + Send,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
{
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> I {
        (*self)(event, ctx)
    }
}
