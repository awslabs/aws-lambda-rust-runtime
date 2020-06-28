use lambda::{handler_fn, Context};
use serde_json::Value;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    Ok(event)
}
