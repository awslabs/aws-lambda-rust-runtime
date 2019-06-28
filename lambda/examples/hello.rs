#![feature(async_await)]

use lambda::{lambda, Error};

#[lambda]
#[runtime::main]
async fn main(event: String) -> Result<String, Error> {
    Ok(event)
}
