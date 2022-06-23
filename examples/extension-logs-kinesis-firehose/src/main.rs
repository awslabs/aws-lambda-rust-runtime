use aws_sdk_firehose::{model::Record, types::Blob, Client};
use lambda_extension::{Error, Extension, LambdaLog, LambdaLogRecord, Service, SharedService};
use std::{future::Future, pin::Pin, task::Poll};

#[derive(Clone)]
struct FirehoseLogsProcessor {
    client: Client,
}

impl FirehoseLogsProcessor {
    pub fn new(client: Client) -> Self {
        FirehoseLogsProcessor { client }
    }
}

/// Implementation of the actual log processor
///
/// This receives a `Vec<LambdaLog>` whenever there are new log entries available.
impl Service<Vec<LambdaLog>> for FirehoseLogsProcessor {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, logs: Vec<LambdaLog>) -> Self::Future {
        let mut records = Vec::with_capacity(logs.len());

        for log in logs {
            match log.record {
                LambdaLogRecord::Function(record) => {
                    records.push(Record::builder().data(Blob::new(record.as_bytes())).build())
                }
                _ => unreachable!(),
            }
        }

        let fut = self
            .client
            .put_record_batch()
            .set_records(Some(records))
            .delivery_stream_name(std::env::var("KINESIS_DELIVERY_STREAM").unwrap())
            .send();

        Box::pin(async move {
            let _ = fut.await?;
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let logs_processor = SharedService::new(FirehoseLogsProcessor::new(Client::new(&config)));

    Extension::new()
        .with_log_types(&["function"])
        .with_logs_processor(logs_processor)
        .run()
        .await?;

    Ok(())
}
