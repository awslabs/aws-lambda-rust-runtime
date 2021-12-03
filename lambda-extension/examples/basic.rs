use lambda_extension::{extension_fn, Error, NextEvent};
use log::LevelFilter;
use simple_logger::SimpleLogger;

async fn my_extension(event: NextEvent) -> Result<(), Error> {
    match event {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        NextEvent::Invoke(_e) => {
            // do something with the invoke event
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let func = extension_fn(my_extension);
    lambda_extension::run(func).await
}
