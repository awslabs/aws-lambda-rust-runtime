use aws_lambda_events::event::sqs::SqsEventObj;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// Object that you send to SQS and plan to process on the function.
#[derive(Deserialize, Serialize)]
struct Data {
    id: String,
    text: String,
}

/// This is the main body for the function.
/// You can use the data sent into SQS here.
async fn function_handler(event: LambdaEvent<SqsEventObj<Data>>) -> Result<(), Error> {
    let data = &event.payload.records[0].body;
    tracing::info!(id = ?data.id, text = ?data.text, "data received from SQS");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
