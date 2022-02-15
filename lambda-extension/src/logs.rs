use serde::{Deserialize, Serialize};
use std::{
    boxed::Box,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

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

impl From<hyper::Request<hyper::Body>> for LambdaLog {
    fn from(_request: hyper::Request<hyper::Body>) -> Self {
        // Todo: implement this
        todo!()
    }
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

/// Service to convert hyper request into a LambdaLog struct
pub(crate) struct LogAdapter<'a, S> {
    service: S,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a, S> LogAdapter<'a, S> {
    /// Create a new LogAdapter
    pub(crate) fn new(service: S) -> Self {
        Self {
            service,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, S> Service<hyper::Request<hyper::Body>> for LogAdapter<'a, S>
where
    S: Service<LambdaLog, Response = ()>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'a,
{
    type Response = hyper::Response<hyper::Body>;
    type Error = S::Error;
    type Future = TransformResponse<'a, S::Error>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        let fut = self.service.call(req.into());
        TransformResponse { fut: Box::pin(fut) }
    }
}

/// Future that transforms a LambdaLog into a hyper response
pub(crate) struct TransformResponse<'a, E> {
    fut: Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>,
}

impl<'a, E> Future for TransformResponse<'a, E> {
    type Output = Result<hyper::Response<hyper::Body>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.fut.as_mut().poll(cx) {
            Poll::Ready(result) => Poll::Ready(result.map(|_| hyper::Response::new(hyper::Body::empty()))),
            Poll::Pending => Poll::Pending,
        }
    }
}
