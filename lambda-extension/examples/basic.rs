use lambda_extension::{run, Error, Extension, InvokeEvent, ShutdownEvent};
use log::LevelFilter;
use simple_logger::SimpleLogger;

struct BasicExtension {}

#[async_trait::async_trait]
impl Extension for BasicExtension {
    async fn on_invoke(&self, _extension_id: &str, _event: InvokeEvent) -> Result<(), Error> {
        Ok(())
    }

    async fn on_shutdown(&self, _extension_id: &str, _event: ShutdownEvent) -> Result<(), Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    run(BasicExtension {}).await?;
    Ok(())
}
