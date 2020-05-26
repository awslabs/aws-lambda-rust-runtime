use lambda_http::{lambda, Error, IntoResponse, Request};

#[lambda(http)]
#[tokio::main]
async fn main(_: Request) -> Result<impl IntoResponse, Error> {
    Ok("👋 world")
}
