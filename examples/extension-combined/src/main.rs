use lambda_extension::{
    service_fn, Error, Extension, LambdaEvent, LambdaLog, LambdaLogRecord, NextEvent, SharedService,
};
use tracing::info;

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

async fn my_log_processor(logs: Vec<LambdaLog>) -> Result<(), Error> {
    for log in logs {
        match log.record {
            LambdaLogRecord::Function(record) => info!("[logs] [function] {}", record),
            LambdaLogRecord::Extension(record) => info!("[logs] [extension] {}", record),
            _ => (),
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
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(my_extension);
    let logs_processor = SharedService::new(service_fn(my_log_processor));

    Extension::new()
        .with_events_processor(func)
        .with_logs_processor(logs_processor)
        .run()
        .await
}
