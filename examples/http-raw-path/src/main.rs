use lambda_http::{service_fn, tracing, Error, IntoResponse, Request, RequestExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {
    let res = format!("The raw path for this request is: {}", event.raw_http_path())
        .into_response()
        .await;

    Ok(res)
}
