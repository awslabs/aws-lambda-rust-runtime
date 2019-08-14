#![feature(async_await, start)]

use lambda::lambda;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(s: String) -> Result<String, Error> {
    Ok(s)
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    0
}