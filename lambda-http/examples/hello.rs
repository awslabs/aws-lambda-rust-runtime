use lambda_http::{lambda_http, Request, IntoResponse};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda_http]
#[tokio::main]
async fn main(_: Request) -> Result<impl IntoResponse, Error> {
    Ok("👋 world")
}