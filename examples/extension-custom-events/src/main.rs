use lambda_extension::{service_fn, tracing, Error, Extension, LambdaEvent, NextEvent};

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
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    Extension::new()
        .with_events(&["SHUTDOWN"])
        .with_events_processor(service_fn(my_extension))
        .run()
        .await
}
