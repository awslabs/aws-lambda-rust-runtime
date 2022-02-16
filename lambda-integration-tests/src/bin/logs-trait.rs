#![allow(clippy::type_complexity)]

use lambda_extension::{Error, Extension, LambdaLog, LambdaLogRecord, Service};
use std::{
    future::{ready, Future},
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc,
    },
    task::Poll,
};
use tracing::info;

#[derive(Default)]
struct MyLogsProcessor {
    counter: Arc<AtomicUsize>,
}

impl MyLogsProcessor {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Implements MakeService for MyLogsProcessor
///
/// The runtime may spawn multiple services, so we need to implement MakeService
/// to have a factory of services. You do that by creating a `Service` that returns
/// a `Service`.
///
/// For this example, we are using the same type for both the service factory and the
/// service itself.
impl Service<()> for MyLogsProcessor {
    type Response = MyLogsProcessor;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        info!("[logs] creating new service");

        Box::pin(ready(Ok(MyLogsProcessor {
            counter: self.counter.clone(),
        })))
    }
}

/// Implementation of the actual log processor
///
/// This receives a `Vec<LambdaLog>` whenever there are new log entries available.
impl Service<Vec<LambdaLog>> for MyLogsProcessor {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, logs: Vec<LambdaLog>) -> Self::Future {
        let counter = self.counter.fetch_add(1, SeqCst);
        for log in logs {
            match log.record {
                LambdaLogRecord::Function(record) => info!("[logs] [function] {}: {}", counter, record.trim()),
                LambdaLogRecord::Extension(record) => info!("[logs] [extension] {}: {}", counter, record.trim()),
                _ => (),
            }
        }

        Box::pin(ready(Ok(())))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    Extension::new()
        .with_logs_processor(MyLogsProcessor::new())
        .run()
        .await?;

    Ok(())
}
