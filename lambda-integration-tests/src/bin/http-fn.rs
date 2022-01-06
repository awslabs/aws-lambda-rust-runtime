use lambda_http::{
    lambda_runtime::{self, Context, Error},
    IntoResponse, Request, Response,
};
use tracing::info;

async fn handler(event: Request, _context: Context) -> Result<impl IntoResponse, Error> {
    info!("[http-fn] Received event {} {}", event.method(), event.uri().path());

    Ok(Response::builder().status(200).body("Hello, world!").unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_runtime::run(lambda_http::handler(handler)).await
}
