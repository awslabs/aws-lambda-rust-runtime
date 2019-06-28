#![feature(async_await)]

use lambda::{lambda, LambdaCtx, Error};

#[lambda]
#[runtime::main]
async fn main(event: String, ctx: LambdaCtx) -> Result<String, Error> {
    let _ = ctx;
    Ok(event)
}
