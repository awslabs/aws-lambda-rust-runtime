use lambda_runtime::{
    service_fn,
    streaming::{channel, Body, Response},
    Error, LambdaEvent,
};
use serde_json::Value;
use std::{thread, time::Duration};

async fn func(_event: LambdaEvent<Value>) -> Result<Response<Body>, Error> {
    let messages = vec!["Hello", "world", "from", "Lambda!"];

    let (mut tx, rx) = channel();

    tokio::spawn(async move {
        for message in messages.iter() {
            tx.send_data((message.to_string() + "\n").into()).await.unwrap();
            thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(Response::from(rx))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_runtime::run(service_fn(func)).await?;
    Ok(())
}
