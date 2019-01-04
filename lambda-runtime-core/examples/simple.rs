use failure::format_err;
use lambda_runtime_core::{lambda, Context, HandlerError};
use simple_logger;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(data: Vec<u8>, _c: Context) -> Result<Vec<u8>, HandlerError> {
    let first_name = String::from_utf8(data)?;

    if first_name == "" {
        return Err(format_err!("First name must be valid").into());
    }

    Ok(format!("Hello, {}!", first_name).as_bytes().to_vec())
}
