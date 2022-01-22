use crate::{requests, Error, ExtensionError, LambdaEvent, NextEvent};
use hyper::client::{connect::Connection, HttpConnector};
use lambda_runtime_api_client::Client;
use std::{fmt, future::Future, path::PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
pub use tower::{self, service_fn, Service};
use tracing::trace;

/// The Runtime handles all the incoming extension requests
pub struct Runtime<C: Service<http::Uri> = HttpConnector> {
    extension_id: String,
    client: Client<C>,
}

impl Runtime {
    /// Create a [`RuntimeBuilder`] to initialize the [`Runtime`]
    pub fn builder<'a>() -> RuntimeBuilder<'a> {
        RuntimeBuilder::default()
    }
}

impl<C> Runtime<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    /// Execute the given extension.
    /// Register the extension with the Extensions API and wait for incoming events.
    pub async fn run<E>(&self, mut extension: E) -> Result<(), Error>
    where
        E: Service<LambdaEvent>,
        E::Future: Future<Output = Result<(), E::Error>>,
        E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
    {
        let client = &self.client;

        let incoming = async_stream::stream! {
            loop {
                trace!("Waiting for next event (incoming loop)");
                let req = requests::next_event_request(&self.extension_id)?;
                let res = client.call(req).await;
                yield res;
            }
        };

        tokio::pin!(incoming);
        while let Some(event) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = event?;
            let (_parts, body) = event.into_parts();

            let body = hyper::body::to_bytes(body).await?;
            trace!("{}", std::str::from_utf8(&body)?); // this may be very verbose
            let event: NextEvent = serde_json::from_slice(&body)?;
            let is_invoke = event.is_invoke();

            let event = LambdaEvent {
                extension_id: self.extension_id.clone(),
                next: event,
            };

            let res = extension.call(event).await;
            if let Err(error) = res {
                let req = if is_invoke {
                    requests::init_error(&self.extension_id, &error.to_string(), None)?
                } else {
                    requests::exit_error(&self.extension_id, &error.to_string(), None)?
                };

                self.client.call(req).await?;
                return Err(error.into());
            }
        }

        Ok(())
    }
}

/// Builder to construct a new extension [`Runtime`]
#[derive(Default)]
pub struct RuntimeBuilder<'a> {
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
}

impl<'a> RuntimeBuilder<'a> {
    /// Create a new [`RuntimeBuilder`] with a given extension name
    pub fn with_extension_name(self, extension_name: &'a str) -> Self {
        RuntimeBuilder {
            extension_name: Some(extension_name),
            ..self
        }
    }

    /// Create a new [`RuntimeBuilder`] with a list of given events.
    /// The only accepted events are `INVOKE` and `SHUTDOWN`.
    pub fn with_events(self, events: &'a [&'a str]) -> Self {
        RuntimeBuilder {
            events: Some(events),
            ..self
        }
    }

    /// Initialize and register the extension in the Extensions API
    pub async fn register(&self) -> Result<Runtime, Error> {
        let name = match self.extension_name {
            Some(name) => name.into(),
            None => {
                let args: Vec<String> = std::env::args().collect();
                PathBuf::from(args[0].clone())
                    .file_name()
                    .expect("unexpected executable name")
                    .to_str()
                    .expect("unexpect executable name")
                    .to_string()
            }
        };

        let events = self.events.unwrap_or(&["INVOKE", "SHUTDOWN"]);

        let client = Client::builder().build()?;

        let req = requests::register_request(&name, events)?;
        let res = client.call(req).await?;
        if res.status() != http::StatusCode::OK {
            return Err(ExtensionError::boxed("unable to register the extension"));
        }

        let extension_id = res.headers().get(requests::EXTENSION_ID_HEADER).unwrap().to_str()?;
        Ok(Runtime {
            extension_id: extension_id.into(),
            client,
        })
    }
}
