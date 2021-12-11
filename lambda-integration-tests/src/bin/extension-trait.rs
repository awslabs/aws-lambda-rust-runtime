use lambda_extension::{Extension, Error, NextEvent};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::{
    future::{ready, Future},
    pin::Pin,
};

struct MyExtension {
    invoke_count: usize,
}

impl Default for MyExtension {
    fn default() -> Self {
        Self { invoke_count: 0 }
    }
}

impl Extension for MyExtension {
    type Fut = Pin<Box<dyn Future<Output = Result<(), Error>>>>;

    fn call(&mut self, event: NextEvent) -> Self::Fut {
        match event {
            NextEvent::Shutdown(e) => {
                info!("[extension] Shutdown event received: {:?}", e);
            },
            NextEvent::Invoke(e) => {
                self.invoke_count += 1;
                info!("[extension] Request event {} received: {:?}", self.invoke_count, e);
            },
        }

        Box::pin(ready(Ok(())))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    lambda_extension::run(MyExtension::default()).await
}