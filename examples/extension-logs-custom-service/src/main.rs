use lambda_extension::{Error, Extension, LambdaLog, LambdaLogRecord, Service, SharedService};
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

/// Custom log processor that increments a counter for each log record.
///
/// This is a simple example of a custom log processor that can be used to
/// count the number of log records that are processed.
///
/// This needs to derive Clone (and store the counter in an Arc) as the runtime
/// could need multiple `Service`s to process the logs.
#[derive(Clone, Default)]
struct MyLogsProcessor {
    counter: Arc<AtomicUsize>,
}

impl MyLogsProcessor {
    pub fn new() -> Self {
        Self::default()
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
                LambdaLogRecord::Function(record) => info!("[logs] [function] {}: {}", counter, record),
                LambdaLogRecord::Extension(record) => info!("[logs] [extension] {}: {}", counter, record),
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
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let logs_processor = SharedService::new(MyLogsProcessor::new());

    Extension::new().with_logs_processor(logs_processor).run().await?;

    Ok(())
}
