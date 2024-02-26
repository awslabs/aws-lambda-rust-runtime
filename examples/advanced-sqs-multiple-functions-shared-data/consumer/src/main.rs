use aws_lambda_events::event::sqs::SqsEventObj;
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use pizza_lib::Pizza;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<SqsEventObj<Pizza>>) -> Result<(), Error> {
    for record in event.payload.records.iter() {
        let pizza = &record.body;
        tracing::info!(name = pizza.name, toppings = ?pizza.toppings, "pizza order received");
    }
    Ok(())
}
