use lambda_extension::{extension_fn, Error, NextEvent};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

async fn my_extension(event: NextEvent) -> Result<(), Error> {
    match event {
        NextEvent::Shutdown(e) => {
            info!("[extension-fn] Shutdown event received: {:?}", e);
        },
        NextEvent::Invoke(e) => {
            info!("[extension-fn] Request event received: {:?}", e);
        },
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    lambda_extension::run(extension_fn(my_extension)).await
}