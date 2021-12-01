use lambda_extension::{run, Error, Extension, InvokeEvent, ShutdownEvent, requests::exit_error, requests::init_error};
use lambda_runtime_api_client::Client;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[derive(Debug)]
enum ErrorExample {
    OnInvokeError,
    OnShutdownError,
}

impl std::error::Error for ErrorExample {}

impl std::fmt::Display for ErrorExample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
          ErrorExample::OnInvokeError => write!(f, "error processing invocation call"),
          ErrorExample::OnShutdownError => write!(f, "error processing shutdown call"),
        }
      }
}

struct ErrorHandlingExtension {
    client: Client
}

#[async_trait::async_trait]
impl Extension for ErrorHandlingExtension {
    async fn on_invoke(&self, extension_id: &str, _event: InvokeEvent) -> Result<(), Error> {
        let err = ErrorExample::OnInvokeError;
        let req = init_error(extension_id, &format!("{}", err), None)?;
        self.client.call(req).await?;
        Err(Box::new(err))
    }

    async fn on_shutdown(&self, extension_id: &str, _event: ShutdownEvent) -> Result<(), Error> {
        let err = ErrorExample::OnShutdownError;
        let req = exit_error(extension_id, &format!("{}", err), None)?;
        self.client.call(req).await?;
        Err(Box::new(err))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let client = Client::builder().build()?;
    let extension = ErrorHandlingExtension { client };

    run(extension).await?;
    Ok(())
}
