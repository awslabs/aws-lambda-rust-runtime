use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};

async fn my_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
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
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(my_extension);
    lambda_extension::run(func).await
}
