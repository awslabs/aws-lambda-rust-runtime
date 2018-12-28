extern crate failure;
extern crate lambda_runtime as lambda;
extern crate log;
extern crate serde_derive;
extern crate simple_logger;

use failure::Fail;
use lambda::{error::HandlerError, lambda};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::error::Error as StdError;

#[derive(Fail, Debug)]
#[fail(display = "Custom Error")]
struct CustomError;

#[derive(Deserialize)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
    age: String,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err((CustomError {}).into());
    }

    let _age_num: u8 = e.age.parse()?;

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
