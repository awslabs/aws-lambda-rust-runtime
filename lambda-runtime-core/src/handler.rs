use crate::context::Context;
use failure::Fail;
use lambda_runtime_errors::LambdaErrorExt;
use std::fmt::Display;

/// Functions acting as a handler must conform to this type.
pub trait Handler<E> {
    /// Run the handler.
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, E>;
}

impl<'ev, F, E> Handler<E> for F
where
    F: FnMut(Vec<u8>, Context) -> Result<Vec<u8>, E>,
    E: Fail + LambdaErrorExt + Display + Send + Sync,
{
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, E> {
        (*self)(event, ctx)
    }
}
