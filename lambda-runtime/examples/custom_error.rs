use lambda_runtime::{error::LambdaErrorExt, lambda, Context};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug)]
struct CustomError {
    msg: String,
}
impl CustomError {
    fn new(message: &str) -> CustomError {
        CustomError {
            msg: message.to_owned(),
        }
    }
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for CustomError {}
impl From<std::num::ParseIntError> for CustomError {
    fn from(i: std::num::ParseIntError) -> Self {
        CustomError::new(&format!("{}", i))
    }
}
// the value return by the error_type function is included as the
// `errorType` in the AWS Lambda response
impl LambdaErrorExt for CustomError {
    fn error_type(&self) -> &str {
        "CustomError"
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

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: CustomEvent, c: Context) -> Result<CustomOutput, CustomError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        // in this case, we explicitly initialize and box our custom error type.
        // the HandlerError type is an alias to Box<dyn Error>/
        return Err(CustomError::new("Empty first name"));
    }

    // For errors simply want to return, because the HandlerError is an alias to any
    // generic error type, we can propapgate with the standard "?" syntax.
    let _age_num: u8 = e.age.parse()?;

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
