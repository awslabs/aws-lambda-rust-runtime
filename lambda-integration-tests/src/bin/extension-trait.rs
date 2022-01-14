use lambda_extension::{Error, Service, LambdaEvent, NextEvent};
use std::{
    future::{ready, Future},
    pin::Pin,
};
use tracing::info;

#[derive(Default)]
struct MyExtension {
    invoke_count: usize,
}

impl Service<LambdaEvent> for MyExtension {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    type Response = ();

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: LambdaEvent) -> Self::Future {
        match event.next {
            NextEvent::Shutdown(e) => {
                info!("[extension] Shutdown event received: {:?}", e);
            }
            NextEvent::Invoke(e) => {
                self.invoke_count += 1;
                info!("[extension] Request event {} received: {:?}", self.invoke_count, e);
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
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_extension::run(MyExtension::default()).await
}
