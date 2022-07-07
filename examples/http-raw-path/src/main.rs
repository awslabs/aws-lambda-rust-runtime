use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {
    let res = format!("The raw path for this request is: {}", event.raw_http_path())
        .into_response()
        .await;

    Ok(res)
}
