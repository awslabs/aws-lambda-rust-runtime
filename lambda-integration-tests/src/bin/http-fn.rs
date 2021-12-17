use lambda_http::{
    lambda_runtime::{self, Context, Error},
    IntoResponse, Request, Response,
};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

async fn handler(event: Request, _context: Context) -> Result<impl IntoResponse, Error> {
    info!("[http-fn] Received event {} {}", event.method(), event.uri().path());

    Ok(Response::builder().status(200).body("Hello, world!").unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    lambda_runtime::run(lambda_http::handler(handler)).await
}
