#![feature(async_await)]

use lambda::{lambda, LambdaCtx};
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: String, ctx: LambdaCtx) -> Result<String, Error> {
    let _ = ctx;
    Ok(event)
}
