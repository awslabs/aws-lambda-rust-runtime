use std::{
    future::{ready, Future},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};

use lambda_runtime::{Error, LambdaEvent, Service};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Debug)]
struct Request {
    command: String,
}

#[derive(Serialize, Debug)]
struct Response {
    message: String,
}

struct MyHandler {
    invoke_count: usize,
    ready: AtomicBool,
}

impl Default for MyHandler {
    fn default() -> Self {
        Self {
            invoke_count: usize::default(),
            // New instances are not ready to be called until polled.
            ready: false.into(),
        }
    }
}

impl Clone for MyHandler {
    fn clone(&self) -> Self {
        Self {
            invoke_count: self.invoke_count,
            // Cloned instances may not be immediately ready to be called.
            // https://docs.rs/tower/0.4.13/tower/trait.Service.html#be-careful-when-cloning-inner-services
            ready: false.into(),
        }
    }
}

impl Service<LambdaEvent<Request>> for MyHandler {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>>>>;
    type Response = Response;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        if self.ready.swap(true, Ordering::SeqCst) {
            info!("[runtime-trait] Service was already ready");
        } else {
            info!("[runtime-trait] Service is now ready");
        };

        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: LambdaEvent<Request>) -> Self::Future {
        self.invoke_count += 1;
        info!("[runtime-trait] Received event {}: {:?}", self.invoke_count, request);

        // After being called once, the service is no longer ready until polled again.
        if self.ready.swap(false, Ordering::SeqCst) {
            info!("[runtime-trait] The service is ready");
        } else {
            // https://docs.rs/tower/latest/tower/trait.Service.html#backpressure
            // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
            // > Services are permitted to panic if `call` is invoked without obtaining
            // > `Poll::Ready(Ok(()))` from `poll_ready`.
            panic!("[runtime-trait] The service is not ready; `.poll_ready()` must be called first");
        }

        Box::pin(ready(Ok(Response {
            message: request.payload.command.to_uppercase(),
        })))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_runtime::run(MyHandler::default()).await
}
