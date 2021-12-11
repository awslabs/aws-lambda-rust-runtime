use lambda_extension::{run, Error, Extension, InvokeEvent, NextEvent};
use log::LevelFilter;
use simple_logger::SimpleLogger;
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
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    run(MyExtension::default()).await
}
