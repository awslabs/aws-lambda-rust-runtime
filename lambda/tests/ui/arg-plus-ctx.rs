#![feature(async_await)]

use lambda::{lambda, LambdaCtx, Error};

#[lambda]
#[runtime::main]
async fn main(s: String, ctx: LambdaCtx) -> Result<String, Error> {
    let _ = ctx;
    Ok(s)
}
