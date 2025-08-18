//! # Example: Axum Streaming Responses on AWS Lambda
//!
//! Demonstrates serving **incremental streaming responses** from Axum handlers
//! running in AWS Lambda.
//!
//! - Streams numbers if `USE_NUMBERS` is set, otherwise streams words.
//! - Uses `BoxService` to erase the router's concrete type so different routers
//!   can be selected at runtime.
//! - Runs with `run_with_streaming_response`, which uses the **default Lambda
//!   runtime** to convert Axum responses into streaming bodies delivered as
//!   data is produced (unlike the OTel example, which used a custom `Runtime` +
//!   `StreamAdapter`).

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
use lambda_http::{run_with_streaming_response, tower::util::BoxService, tracing, Error};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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

async fn stream_numbers() -> Result<Response, AppError> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(8);
    let body = Body::from_stream(ReceiverStream::new(rx));

    tokio::spawn(async move {
        for i in 1..=4 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx.send(Ok(Bytes::from(format!("number: {i}\n")))).await;
        }
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(CACHE_CONTROL, "no-cache")
        .body(body)?)
}

async fn stream_words() -> Result<Response, AppError> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(8);
    let body = Body::from_stream(ReceiverStream::new(rx));

    tokio::spawn(async move {
        for msg in ["Hello", "world", "from", "Lambda!"] {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if tx.send(Ok(Bytes::from(format!("{msg}\n")))).await.is_err() {
                break;
            }
        }
    });

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
    tracing::init_default_subscriber();

    let svc = create_svc();

    // Automatically convert the service into a streaming response with a
    // default runtime.
    run_with_streaming_response(svc).await
}
