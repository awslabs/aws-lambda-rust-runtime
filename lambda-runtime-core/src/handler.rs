use crate::context::Context;
use std::{fmt::Display, fmt::Debug, fmt};

/// Functions acting as a handler must conform to this type.
pub trait Handler {
    /// Run the handler.
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, HandlerError>;
}

impl<'ev, F> Handler for F
where
    F: FnMut(Vec<u8>, Context) -> Result<Vec<u8>, HandlerError>,
{
    fn run(&mut self, event: Vec<u8>, ctx: Context) -> Result<Vec<u8>, HandlerError> {
        (*self)(event, ctx)
    }
}

/// The `HandlerError` struct can be use to abstract any `Err` of the handler method `Result`.
/// The `HandlerError` object can be generated `From` any object that supports `Display`,
/// `Send, `Sync`, and `Debug`. This allows handler functions to return any error using
/// the `?` syntax. For example `let _age_num: u8 = e.age.parse()?;` will return the
/// `<F as FromStr>::Err` from the handler function.
pub struct HandlerError {
    msg: String,
}

impl<E: Display + Send + Sync + Debug> From<E> for HandlerError {
    fn from(e: E) -> HandlerError {
        HandlerError { msg: format!("{}", e) }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}