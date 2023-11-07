use lambda_runtime::{service_fn, Error, LambdaEvent};
use pizza_lib::Pizza;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_: LambdaEvent<Value>) -> Result<(), Error> {
    // read the queue url from the environment
    let queue_url = std::env::var("QUEUE_URL").expect("could not read QUEUE_URL");

    // let's create our pizza
    let message = Pizza {
        name: "margherita".to_string(),
        toppings: vec![
            "San Marzano Tomatoes".to_string(),
            "Fresh Mozzarella".to_string(),
            "Basil".to_string(),
        ],
    };

    // create our SQS client
    let config = aws_config::from_env().load().await;

    // send our message to SQS
    let client = aws_sdk_sqs::Client::new(&config);
    client
        .send_message()
        .queue_url(queue_url)
        .message_body(json!(message).to_string())
        .send()
        .await?;

    Ok(())
}
