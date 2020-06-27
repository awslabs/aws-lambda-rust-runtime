use lambda_http::{
    lambda::{lambda, LambdaCtx},
    IntoResponse, Request,
};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda(http)]
#[tokio::main]
async fn main(_: Request, _: LambdaCtx) -> Result<impl IntoResponse, Error> {
    Ok("👋 world")
}
