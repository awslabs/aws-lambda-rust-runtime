use crate::context::Context;
use failure::Fail;
use lambda_runtime_errors::LambdaErrorExt;
use std::fmt::Display;

/// Functions acting as a handler must conform to this type.
pub trait Handler<EventError>: Send {
    /// Run the handler.
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, EventError>;
}

impl<'ev, Function, EventError> Handler<EventError> for Function
where
    Function: FnMut(Vec<u8>, Context) -> Result<Vec<u8>, EventError> + Send,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
{
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, EventError> {
        (*self)(event, ctx)
    }
}
