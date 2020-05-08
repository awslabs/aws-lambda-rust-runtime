use lambda_http::{lambda, IntoResponse, Request};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda(http)]
#[tokio::main]
async fn main(_: Request) -> Result<impl IntoResponse, Error> {
    Ok("👋 world")
}
