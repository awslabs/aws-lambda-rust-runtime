use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use pizza_lib::Pizza;
use aws_lambda_json_impl::{json, Value};

struct SQSManager {
    client: aws_sdk_sqs::Client,
    queue_url: String,
}

impl SQSManager {
    fn new(client: aws_sdk_sqs::Client, queue_url: String) -> Self {
        Self { client, queue_url }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    // read the queue url from the environment
    let queue_url = std::env::var("QUEUE_URL").expect("could not read QUEUE_URL");
    // build the config from environment variables (fed by AWS Lambda)
    let config = aws_config::from_env().load().await;
    // create our SQS Manager
    let sqs_manager = SQSManager::new(aws_sdk_sqs::Client::new(&config), queue_url);
    let sqs_manager_ref = &sqs_manager;

    // no need to create a SQS Client for each incoming request, let's use a shared state
    let handler_func_closure = |event: LambdaEvent<Value>| async move { process_event(event, sqs_manager_ref).await };
    lambda_runtime::run(service_fn(handler_func_closure)).await?;
    Ok(())
}

async fn process_event(_: LambdaEvent<Value>, sqs_manager: &SQSManager) -> Result<(), Error> {
    // let's create our pizza
    let message = Pizza {
        name: "margherita".to_string(),
        toppings: vec![
            "San Marzano Tomatoes".to_string(),
            "Fresh Mozzarella".to_string(),
            "Basil".to_string(),
        ],
    };
    // send our message to SQS
    sqs_manager
        .client
        .send_message()
        .queue_url(&sqs_manager.queue_url)
        .message_body(json!(message).to_string())
        .send()
        .await?;

    Ok(())
}
