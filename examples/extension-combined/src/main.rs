use lambda_extension::{
    service_fn, tracing, Error, Extension, LambdaEvent, LambdaLog, LambdaLogRecord, NextEvent, SharedService,
};

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
            LambdaLogRecord::Function(record) => tracing::info!("[logs] [function] {}", record),
            LambdaLogRecord::Extension(record) => tracing::info!("[logs] [extension] {}", record),
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let func = service_fn(my_extension);
    let logs_processor = SharedService::new(service_fn(my_log_processor));

    Extension::new()
        .with_events_processor(func)
        .with_logs_processor(logs_processor)
        .run()
        .await
}
