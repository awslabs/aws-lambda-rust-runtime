use lambda_extension::{run, tracing, Error, InvokeEvent, LambdaEvent, NextEvent, Service};
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
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run(MyExtension::default()).await
}
