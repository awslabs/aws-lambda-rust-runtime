use lambda_runtime::{layers::OpenTelemetryLayer as OtelLayer, LambdaEvent, Runtime};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{runtime, trace};
use tower::{service_fn, BoxError};
use tracing_subscriber::prelude::*;

async fn echo(event: LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, &'static str> {
    Ok(event.payload)
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Set up OpenTelemetry tracer provider that writes spans to stdout for debugging purposes
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let tracer_provider = trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    // Set up link between OpenTelemetry and tracing crate
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::OpenTelemetryLayer::new(
            tracer_provider.tracer("my-app"),
        ))
        .init();

    // Initialize the Lambda runtime and add OpenTelemetry tracing
    let runtime = Runtime::new(service_fn(echo)).layer(OtelLayer::new(|| {
        // Make sure that the trace is exported before the Lambda runtime is frozen
        tracer_provider.force_flush();
    }));
    runtime.run().await?;
    Ok(())
}
