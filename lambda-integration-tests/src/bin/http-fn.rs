use lambda_http::{ext::RequestExt, service_fn, Body, Error, IntoResponse, Request, Response};
use tracing::info;

async fn handler(event: Request) -> Result<impl IntoResponse, Error> {
    let _context = event.lambda_context();
    info!("[http-fn] Received event {} {}", event.method(), event.uri().path());

    Ok(Response::builder()
        .status(200)
        .body(Body::from("Hello, world!"))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let handler = service_fn(handler);
    lambda_http::run(handler).await
}
