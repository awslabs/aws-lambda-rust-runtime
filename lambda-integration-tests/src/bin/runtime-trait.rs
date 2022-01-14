use lambda_runtime::{Error, LambdaEvent, Service};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Future},
    pin::Pin,
};
use tracing::info;

#[derive(Deserialize, Debug)]
struct Request {
    command: String,
}

#[derive(Serialize, Debug)]
struct Response {
    message: String,
}

#[derive(Default)]
struct MyHandler {
    invoke_count: usize,
}

impl Service<LambdaEvent<Request>> for MyHandler {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>>>>;
    type Response = Response;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: LambdaEvent<Request>) -> Self::Future {
        self.invoke_count += 1;
        info!("[handler] Received event {}: {:?}", self.invoke_count, request);
        Box::pin(ready(Ok(Response {
            message: request.event.command.to_uppercase(),
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
