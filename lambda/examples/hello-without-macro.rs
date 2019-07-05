#![feature(async_await)]

use lambda::{handler_fn, LambdaCtx};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[runtime::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: String, _: Option<LambdaCtx>) -> Result<String, Error> {
    Ok(event)
}
