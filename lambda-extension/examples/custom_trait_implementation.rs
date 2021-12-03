use lambda_extension::{run, Error, NextEvent, Extension};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::future::{Future, ready};
use std::pin::Pin;

struct MyExtension {}

impl Extension for MyExtension
{
    type Fut = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    fn call(&mut self, event: NextEvent) -> Self::Fut {
        match event {
            NextEvent::Shutdown(_e) => {
                // do something with the shutdown event
            }
            _ => {
                // ignore any other event
                // because we've registered the extension
                // only to receive SHUTDOWN events
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

    run(MyExtension {}).await
}
