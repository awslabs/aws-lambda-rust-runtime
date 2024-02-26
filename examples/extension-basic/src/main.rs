use lambda_extension::{service_fn, tracing, Error, LambdaEvent, NextEvent};

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
    tracing::init_default_subscriber();

    let func = service_fn(my_extension);
    lambda_extension::run(func).await
}
