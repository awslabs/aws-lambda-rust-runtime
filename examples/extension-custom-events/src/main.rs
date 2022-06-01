use lambda_extension::{service_fn, Error, Extension, LambdaEvent, NextEvent};

async fn my_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        _ => {
            // ignore any other event
            // because we've registered the extension
            // only to receive SHUTDOWN events
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

    Extension::new()
        .with_events(&["SHUTDOWN"])
        .with_events_processor(service_fn(my_extension))
        .run()
        .await
}
