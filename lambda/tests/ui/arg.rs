#![feature(async_await)]

use lambda::{lambda, Error};

#[lambda]
#[runtime::main]
async fn main(s: String) -> Result<String, Error> {
    Ok(s)
}
