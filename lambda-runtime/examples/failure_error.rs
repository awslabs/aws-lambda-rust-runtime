use failure::{format_err, Compat, Error};
use lambda_runtime::{error::LambdaResultExt, lambda, Context};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::error::Error as StdError;

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

fn my_handler(e: CustomEvent, c: Context) -> Result<CustomOutput, Compat<Error>> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        let err = format_err!("Invalid First Name");
        return Err(err.compat());
    }

    let _age_num: u8 = e.age.parse().failure_compat()?;

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
