use serde::{Deserialize, Serialize};
use std::{boxed::Box, sync::Arc};
use tokio::sync::Mutex;
use tower::Service;
use tracing::{error, trace};

/// Payload received from the Lambda Logs API
/// See: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-logs-api.html#runtimes-logs-api-msg
#[derive(Clone, Debug, Deserialize)]
pub struct LambdaLog {
    /// Time when the log was generated
    pub time: String,
    /// Log type, either function, extension, or platform types
    pub r#type: String,
    // Fixme(david): the record can be a struct with more information, implement custom deserializer
    /// Log data
    pub record: String,
}

/// Log buffering configuration.
/// Allows Lambda to buffer logs before deliverying them to a subscriber.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogBuffering {
    /// The maximum time (in milliseconds) to buffer a batch.
    /// Default: 1,000. Minimum: 25. Maximum: 30,000
    pub timeout_ms: usize,
    /// The maximum size (in bytes) of the logs to buffer in memory.
    /// Default: 262,144. Minimum: 262,144. Maximum: 1,048,576
    pub max_bytes: usize,
    /// The maximum number of events to buffer in memory.
    /// Default: 10,000. Minimum: 1,000. Maximum: 10,000
    pub max_items: usize,
}

impl Default for LogBuffering {
    fn default() -> Self {
        LogBuffering {
            timeout_ms: 1_000,
            max_bytes: 262_144,
            max_items: 10_000,
        }
    }
}

/// Wrapper function that sends logs to the subscriber Service
///
/// This takes an `hyper::Request` and transforms it into `Vec<LambdaLog>` for the
/// underlying `Service` to process.
pub(crate) async fn log_wrapper<S>(
    service: Arc<Mutex<S>>,
    req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, Box<dyn std::error::Error + Send + Sync>>
where
    S: Service<Vec<LambdaLog>, Response = ()>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    trace!("Received logs request");
    // Parse the request body as a Vec<LambdaLog>
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(body) => body,
        Err(e) => {
            error!("Error reading logs request body: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(hyper::Body::empty())
                .unwrap());
        }
    };
    let logs: Vec<LambdaLog> = match serde_json::from_slice(&body) {
        Ok(logs) => logs,
        Err(e) => {
            error!("Error parsing logs: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(hyper::Body::empty())
                .unwrap());
        }
    };

    {
        let mut service = service.lock().await;
        let _ = service.call(logs).await;
    }

    Ok(hyper::Response::new(hyper::Body::empty()))
}
