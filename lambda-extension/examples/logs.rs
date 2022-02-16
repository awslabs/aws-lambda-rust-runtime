use lambda_extension::{service_fn, Error, Extension, LambdaLog, LambdaLogRecord, SharedService};
use tracing::info;

async fn handler(logs: Vec<LambdaLog>) -> Result<(), Error> {
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
    let logs_processor = SharedService::new(service_fn(handler));

    Extension::new().with_logs_processor(logs_processor).run().await?;

    Ok(())
}
