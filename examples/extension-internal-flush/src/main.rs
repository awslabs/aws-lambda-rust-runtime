use anyhow::anyhow;
use aws_lambda_events::sqs::{SqsBatchResponse, SqsEventObj};
use lambda_extension::{service_fn, Error, Extension, NextEvent};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use std::sync::Arc;

/// Implements an internal Lambda extension to flush logs/telemetry after each request.
struct FlushExtension {
    request_done_receiver: Arc<Mutex<UnboundedReceiver<()>>>,
}

impl FlushExtension {
    pub fn new(request_done_receiver: UnboundedReceiver<()>) -> Self {
        Self {
            request_done_receiver: Arc::new(Mutex::new(request_done_receiver)),
        }
    }

    pub async fn invoke(&self, event: lambda_extension::LambdaEvent) -> Result<(), Error> {
        match event.next {
            // NB: Internal extensions only support the INVOKE event.
            NextEvent::Shutdown(shutdown) => {
                return Err(anyhow!("extension received unexpected SHUTDOWN event: {:?}", shutdown).into());
            }
            NextEvent::Invoke(_e) => {}
        }

        eprintln!("[extension] waiting for event to be processed");

        // Wait for runtime to finish processing event.
        self.request_done_receiver
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| anyhow!("channel is closed"))?;

        eprintln!("[extension] flushing logs and telemetry");

        // <flush logs and telemetry here>

        Ok(())
    }
}

/// Object that you send to SQS and plan to process with the function.
#[derive(Debug, Deserialize, Serialize)]
struct Data {
    a: String,
    b: i64,
}

/// Implements the main event handler for processing events from an SQS queue.
struct EventHandler {
    request_done_sender: Arc<Mutex<UnboundedSender<()>>>,
}

impl EventHandler {
    pub fn new(request_done_sender: UnboundedSender<()>) -> Self {
        Self {
            request_done_sender: Arc::new(Mutex::new(request_done_sender)),
        }
    }

    pub async fn invoke(
        &self,
        event: lambda_runtime::LambdaEvent<SqsEventObj<Data>>,
    ) -> Result<SqsBatchResponse, Error> {
        let data = &event.payload.records[0].body;
        eprintln!("[runtime] received event {data:?}");

        // <process event here>

        // Notify the extension to flush traces.
        self.request_done_sender.lock().await.send(()).map_err(Box::new)?;

        Ok(SqsBatchResponse::default())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (request_done_sender, request_done_receiver) = unbounded_channel::<()>();

    let flush_extension = Arc::new(FlushExtension::new(request_done_receiver));
    let extension = Extension::new()
        // Internal extensions only support INVOKE events.
        .with_events(&["INVOKE"])
        .with_events_processor(service_fn(|event| {
            let flush_extension = flush_extension.clone();
            async move { flush_extension.invoke(event).await }
        }))
        // Extensions MUST be registered before calling lambda_runtime::run(), which ends the Init
        // phase and begins the Invoke phase.
        .register()
        .await?;

    let handler = Arc::new(EventHandler::new(request_done_sender));

    tokio::try_join!(
        lambda_runtime::run(service_fn(|event| {
            let handler = handler.clone();
            async move { handler.invoke(event).await }
        })),
        extension.run(),
    )?;

    Ok(())
}
