use lambda::{lambda, LambdaCtx};
use serde_json::Value;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: Value, _: LambdaCtx) -> Result<Value, Error> {
    Ok(event)
}
