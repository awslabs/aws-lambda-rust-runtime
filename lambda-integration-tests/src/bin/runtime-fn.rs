use lambda_runtime::{handler_fn, Context, Error};
use log::{info, LevelFilter};
use serde::{Serialize, Deserialize};
use simple_logger::SimpleLogger;

#[derive(Deserialize, Debug)]
struct Request {
    command: String,
}

#[derive(Serialize, Debug)]
struct Response {
    message: String,
}

async fn handler(event: Request, _context: Context) -> Result<Response, Error> {
    info!("[handler-fn] Received event: {:?}", event);

    Ok(Response {
        message: event.command.to_uppercase(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    lambda_runtime::run(handler_fn(handler)).await
}