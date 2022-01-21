use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Debug)]
struct Request {
    command: String,
}

#[derive(Serialize, Debug)]
struct Response {
    message: String,
}

async fn handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("[handler-fn] Received event: {:?}", event);

    let (event, _) = event.into_parts();

    Ok(Response {
        message: event.command.to_uppercase(),
    })
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

    lambda_runtime::run(service_fn(handler)).await
}
