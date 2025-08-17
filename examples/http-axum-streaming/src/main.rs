use axum::{
    body::Body,
    http,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bytes::Bytes;
use lambda_http::{lambda_runtime, tracing, Error, StreamAdapter};
use std::{convert::Infallible, time::Duration};
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

type AppResult<T = Response> = Result<T, AppError>;

async fn stream_handler() -> AppResult {
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let app = Router::new().route("/", get(stream_handler));

    let runtime = lambda_runtime::Runtime::new(StreamAdapter::from(app));

    runtime.run().await
}
