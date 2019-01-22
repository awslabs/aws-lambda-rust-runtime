use failure::Fail;
use lambda_runtime::{error::LambdaErrorExt, lambda, Context};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::error::Error as StdError;

#[derive(Fail, Debug)]
#[fail(display = "Custom Error")]
struct CustomError;
impl LambdaErrorExt for CustomError {
    fn error_type(&self) -> &str {
        "CustomError"
    }
}
impl From<std::num::ParseIntError> for CustomError {
    fn from(_i: std::num::ParseIntError) -> Self {
        CustomError {}
    }
}

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
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: CustomEvent, c: Context) -> Result<CustomOutput, CustomError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err(CustomError {});
    }

    let _age_num: u8 = e.age.parse()?;

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
