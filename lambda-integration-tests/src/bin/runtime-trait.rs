use lambda_runtime::{Context, Handler, Error};
use log::{info, LevelFilter};
use serde::{Serialize, Deserialize};
use simple_logger::SimpleLogger;
use std::{
    future::{ready, Future},
    pin::Pin,
};

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
}

impl Default for MyHandler {
    fn default() -> Self {
        Self { invoke_count: 0 }
    }
}

impl Handler<Request, Response> for MyHandler {
    type Error = Error;
    type Fut = Pin<Box<dyn Future<Output = Result<Response, Error>>>>;

    fn call(&mut self, event: Request, _context: Context) -> Self::Fut {
        self.invoke_count += 1;
        info!("[handler] Received event {}: {:?}", self.invoke_count, event);
        Box::pin(ready(
            Ok(Response {
                message: event.command.to_uppercase(),
            }))
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    lambda_runtime::run(MyHandler::default()).await
}