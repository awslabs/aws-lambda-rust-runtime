use lambda_extension::{service_fn, Error, Extension, LambdaTelemetry, LambdaTelemetryRecord, SharedService};
use tracing::info;

async fn handler(events: Vec<LambdaTelemetry>) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::Function(record) => info!("[logs] [function] {}", record),
            LambdaTelemetryRecord::PlatformInitStart {
                initialization_type: _,
                phase: _,
                runtime_version: _,
                runtime_version_arn: _,
            } => info!("[platform] Initialization started"),
            LambdaTelemetryRecord::PlatformInitRuntimeDone {
                initialization_type: _,
                phase: _,
                status: _,
                error_type: _,
                spans: _,
            } => info!("[platform] Initialization finished"),
            LambdaTelemetryRecord::PlatformStart {
                request_id,
                version: _,
                tracing: _,
            } => info!("[platform] Handling of request {} started", request_id),
            LambdaTelemetryRecord::PlatformRuntimeDone {
                request_id,
                status: _,
                error_type: _,
                metrics: _,
                spans: _,
                tracing: _,
            } => info!("[platform] Handling of request {} finished", request_id),
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
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let telemetry_processor = SharedService::new(service_fn(handler));

    Extension::new()
        .with_telemetry_processor(telemetry_processor)
        .run()
        .await?;

    Ok(())
}
