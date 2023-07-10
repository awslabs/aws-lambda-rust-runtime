use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};
use tracing::info;

async fn my_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(e) => {
            info!("[extension-fn] Shutdown event received: {:?}", e);
        }
        NextEvent::Invoke(e) => {
            info!("[extension-fn] Request event received: {:?}", e);
        }
    }

    Ok(())
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

    lambda_extension::run(service_fn(my_extension)).await
}
