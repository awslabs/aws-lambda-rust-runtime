use lambda_extension::{run, Error, Extension, InvokeEvent, NextEvent};
use std::{
    future::{ready, Future},
    pin::Pin,
};

#[derive(Default)]
struct MyExtension {
    data: Vec<InvokeEvent>,
}

impl Extension for MyExtension {
    type Fut = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    fn call(&mut self, event: NextEvent) -> Self::Fut {
        match event {
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
    run(MyExtension::default()).await
}
