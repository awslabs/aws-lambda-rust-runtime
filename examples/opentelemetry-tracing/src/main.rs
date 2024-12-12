use lambda_runtime::{
    layers::{OpenTelemetryFaasTrigger, OpenTelemetryLayer as OtelLayer},
    tracing::Span,
    LambdaEvent, Runtime,
};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{runtime, trace};
use tower::{service_fn, BoxError};
use tracing_subscriber::prelude::*;

async fn echo(event: LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, &'static str> {
    let span = Span::current();
    span.record("otel.kind", "SERVER");
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
    let runtime = Runtime::new(service_fn(echo)).layer(
        // Create a tracing span for each Lambda invocation
        OtelLayer::new(|| {
            // Make sure that the trace is exported before the Lambda runtime is frozen
            tracer_provider.force_flush();
        })
        // Set the "faas.trigger" attribute of the span to "pubsub"
        .with_trigger(OpenTelemetryFaasTrigger::PubSub),
    );
    runtime.run().await?;
    Ok(())
}
