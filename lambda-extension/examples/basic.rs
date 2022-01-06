use lambda_extension::{extension_fn, Error, NextEvent};

async fn my_extension(event: NextEvent) -> Result<(), Error> {
    match event {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        NextEvent::Invoke(_e) => {
            // do something with the invoke event
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
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = extension_fn(my_extension);
    lambda_extension::run(func).await
}
