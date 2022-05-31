use lambda_extension::{run, Error, InvokeEvent, LambdaEvent, NextEvent, Service};
use std::{
    future::{ready, Future},
    pin::Pin,
};

#[derive(Default)]
struct MyExtension {
    data: Vec<InvokeEvent>,
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
            NextEvent::Shutdown(_e) => {
                self.data.clear();
            }
            NextEvent::Invoke(e) => {
                self.data.push(e);
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
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(MyExtension::default()).await
}
