use lambda_extension::{service_fn, tracing, Error, Extension, LambdaTelemetry, LambdaTelemetryRecord, SharedService};

async fn handler(events: Vec<LambdaTelemetry>) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::Function(record) => tracing::info!("[logs] [function] {}", record),
            LambdaTelemetryRecord::PlatformInitStart {
                initialization_type: _,
                phase: _,
                runtime_version: _,
                runtime_version_arn: _,
            } => tracing::info!("[platform] Initialization started"),
            LambdaTelemetryRecord::PlatformInitRuntimeDone {
                initialization_type: _,
                phase: _,
                status: _,
                error_type: _,
                spans: _,
            } => tracing::info!("[platform] Initialization finished"),
            LambdaTelemetryRecord::PlatformStart {
                request_id,
                version: _,
                tracing: _,
            } => tracing::info!("[platform] Handling of request {} started", request_id),
            LambdaTelemetryRecord::PlatformRuntimeDone {
                request_id,
                status: _,
                error_type: _,
                metrics: _,
                spans: _,
                tracing: _,
            } => tracing::info!("[platform] Handling of request {} finished", request_id),
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let telemetry_processor = SharedService::new(service_fn(handler));

    Extension::new()
        .with_telemetry_processor(telemetry_processor)
        .run()
        .await?;

    Ok(())
}
