use lambda::{lambda, Error};
use serde_json::Value;

#[lambda]
#[tokio::main]
async fn main(event: Value) -> Result<Value, Error> {
    Ok(event)
}
