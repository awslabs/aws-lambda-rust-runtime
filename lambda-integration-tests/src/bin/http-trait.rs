use lambda_http::{Error, Request, RequestExt, Response, Service};
use std::{
    future::{ready, Future},
    pin::Pin,
};
use tracing::info;

#[derive(Default)]
struct MyHandler {
    invoke_count: usize,
}

impl Service<Request> for MyHandler {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>> + Send>>;
    type Response = Response<&'static str>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        self.invoke_count += 1;
        info!("[http-trait] Received event {}: {:?}", self.invoke_count, request);
        info!("[http-trait] Lambda context: {:?}", request.lambda_context());
        Box::pin(ready(Ok(Response::builder().status(200).body("Hello, World!").unwrap())))
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

    lambda_http::run(MyHandler::default()).await
}