#![feature(async_await)]

use lambda::lambda;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[runtime::main]
async fn main(event: String) -> Result<String, Error> {
    Ok(event)
}
