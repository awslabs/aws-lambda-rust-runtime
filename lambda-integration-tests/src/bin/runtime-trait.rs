use lambda_runtime::{Context, Error, Handler};
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

impl Handler<Request, Response> for MyHandler {
    type Error = Error;
    type Fut = Pin<Box<dyn Future<Output = Result<Response, Error>>>>;

    fn call(&mut self, event: Request, _context: Context) -> Self::Fut {
        self.invoke_count += 1;
        info!("[handler] Received event {}: {:?}", self.invoke_count, event);
        Box::pin(ready(Ok(Response {
            message: event.command.to_uppercase(),
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
