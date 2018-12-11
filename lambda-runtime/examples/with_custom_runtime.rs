use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self, error};
use serde_derive::{Deserialize, Serialize};
use simple_logger;
use tokio::runtime::Runtime;

#[derive(Deserialize, Clone)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new()?;

    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler, rt);

    Ok(())
}

fn my_handler(e: CustomEvent, c: Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err(c.new_error("Empty first name"));
    }

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
