use lambda::{lambda, Context};
use serde_json::Value;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

// #[lambda] attribute removes the need for boilerplate code
// required by `lambda::run(func).await?` as demonstrated in other
// examples.

#[lambda]
#[tokio::main]
async fn main(event: Value, _: Context) -> Result<Value, Error> {
    Ok(event)
}
