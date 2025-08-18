//! # Example: Axum Streaming Responses on AWS Lambda with OTel
//!
//! Demonstrates serving **incremental streaming responses** from Axum handlers
//! running in AWS Lambda using a **custom** `lambda_runtime::Runtime` with
//! OpenTelemetry (OTel) support.
//!
//! - Streams numbers if `USE_NUMBERS` is set, otherwise streams words.
//! - Uses `BoxService` to erase the router's concrete type so different routers
//!   can be selected at runtime.
//! - Runs with a custom `Runtime` + `StreamAdapter`, which convert Axum
//!   responses into streaming bodies delivered as data is produced (unlike the
//!   default `run_with_streaming_response` helper).

use axum::{
    body::Body,
    extract::Request,
    http::{
        self,
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bytes::Bytes;
use core::{convert::Infallible, time::Duration};
use lambda_http::{
    lambda_runtime::{
        layers::{OpenTelemetryFaasTrigger, OpenTelemetryLayer as OtelLayer},
        tracing::Instrument,
        Runtime,
    },
    tower::util::BoxService,
    tracing, Error, StreamAdapter,
};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing_subscriber::prelude::*;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Http(#[from] http::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[tracing::instrument(skip_all)]
async fn stream_numbers() -> Result<Response, AppError> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(8);
    let body = Body::from_stream(ReceiverStream::new(rx));

    tokio::spawn(
        async move {
            for (idx, i) in (1..=4).enumerate() {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let line = format!("number: {i}\n");
                tracing::info!(chunk.idx = idx, bytes = line.len(), "emit");
                if tx.send(Ok(Bytes::from(line))).await.is_err() {
                    break;
                }
            }
        }
        .instrument(tracing::info_span!("producer.stream_numbers")),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(CACHE_CONTROL, "no-cache")
        .body(body)?)
}

#[tracing::instrument(skip_all)]
async fn stream_words() -> Result<Response, AppError> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(8);
    let body = Body::from_stream(ReceiverStream::new(rx));

    tokio::spawn(
        async move {
            for (idx, msg) in ["Hello", "world", "from", "Lambda!"].iter().enumerate() {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let line = format!("{msg}\n");
                tracing::info!(chunk.idx = idx, bytes = line.len(), "emit");
                if tx.send(Ok(Bytes::from(line))).await.is_err() {
                    break;
                }
            }
        }
        .instrument(tracing::info_span!("producer.stream_words")),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(CACHE_CONTROL, "no-cache")
        .body(body)?)
}

// Creates a dynamic router based on the environment variable. Demonstrating how
// you can type-erase a service
fn create_svc() -> BoxService<Request<lambda_http::Body>, Response<Body>, Infallible> {
    if std::env::var("USE_NUMBERS").as_deref() == Ok("1") {
        BoxService::new(Router::new().route("/", get(stream_numbers)))
    } else {
        BoxService::new(Router::new().route("/", get(stream_words)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up OpenTelemetry tracer provider that writes spans to stdout for
    // debugging purposes
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let tracer_provider = trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    // Set up link between OpenTelemetry and tracing crate
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::OpenTelemetryLayer::new(
            tracer_provider.tracer("my-streaming-app"),
        ))
        .init();

    let svc = create_svc();

    // Initialize the Lambda runtime and add OpenTelemetry tracing
    let runtime = Runtime::new(StreamAdapter::from(svc)).layer(
        OtelLayer::new(|| {
            if let Err(err) = tracer_provider.force_flush() {
                eprintln!("Error flushing traces: {err:#?}");
            }
        })
        .with_trigger(OpenTelemetryFaasTrigger::Http),
    );

    runtime.run().await
}
