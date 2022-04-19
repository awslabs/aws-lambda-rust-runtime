use lambda_http::{service_fn, Error, IntoResponse, Request};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(_event: Request) -> Result<impl IntoResponse, Error> {
    Ok((200, "Hello, world!"))
}
