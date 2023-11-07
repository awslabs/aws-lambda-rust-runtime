use aws_lambda_events::event::sqs::SqsEventObj;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use pizza_lib::Pizza;

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

async fn func(event: LambdaEvent<SqsEventObj<Pizza>>) -> Result<(), Error> {
    for record in event.payload.records.iter() {
        let pizza = &record.body;
        println!("Pizza name: {} with toppings: {:?}", pizza.name, pizza.toppings);
    }
    Ok(())
}
