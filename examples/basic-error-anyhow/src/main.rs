use anyhow::bail;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct Request {}

/// Return anyhow::Result in the main body for the Lambda function.
async fn function_handler(_event: LambdaEvent<Request>) -> anyhow::Result<()> {
    bail!("This is an error message");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async move {
        function_handler(event)
            .await
            .map_err(Into::<Box<dyn std::error::Error>>::into)
    }))
    .await
}
