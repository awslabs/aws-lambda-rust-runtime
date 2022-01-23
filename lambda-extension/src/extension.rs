use crate::{logs::*, requests, Error, ExtensionError, LambdaEvent, NextEvent};
use lambda_runtime_api_client::Client;
use std::{fmt, future::ready, future::Future, path::PathBuf, pin::Pin};
use tokio_stream::StreamExt;
pub use tower::{self, service_fn, Service};
use tracing::trace;

/// An Extension that runs event and log processors
pub struct Extension<'a, E, L> {
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
    events_processor: E,
    log_types: Option<&'a [&'a str]>,
    logs_processor: Option<L>,
    log_buffering: Option<LogBuffering>,
}

impl<'a> Extension<'a, Identity<LambdaEvent>, Identity<LambdaLog>> {
    /// Create a new base [`Extension`] with a no-op events processor
    pub fn new() -> Self {
        Extension {
            extension_name: None,
            events: None,
            events_processor: Identity::new(),
            log_types: None,
            log_buffering: None,
            logs_processor: None,
        }
    }
}

impl<'a, E, L> Extension<'a, E, L>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,

    L: Service<LambdaLog>,
    L::Future: Future<Output = Result<(), L::Error>>,
    L::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
{
    /// Create a new [`Extension`] with a given extension name
    pub fn with_extension_name(self, extension_name: &'a str) -> Self {
        Extension {
            extension_name: Some(extension_name),
            ..self
        }
    }

    /// Create a new [`Extension`] with a list of given events.
    /// The only accepted events are `INVOKE` and `SHUTDOWN`.
    pub fn with_events(self, events: &'a [&'a str]) -> Self {
        Extension {
            events: Some(events),
            ..self
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda events.
    pub fn with_events_processor<N>(self, ep: N) -> Extension<'a, N, L>
    where
        N: Service<LambdaEvent>,
        N::Future: Future<Output = Result<(), N::Error>>,
        N::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
    {
        Extension {
            events_processor: ep,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            logs_processor: self.logs_processor,
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda logs.
    pub fn with_logs_processor<N>(self, lp: N) -> Extension<'a, E, N>
    where
        N: Service<LambdaLog>,
        N::Future: Future<Output = Result<(), N::Error>>,
        N::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
    {
        Extension {
            logs_processor: Some(lp),
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
        }
    }

    /// Create a new [`Extension`] with a list of logs types to subscribe.
    /// The only accepted log types are `function`, `platform`, and `extension`.
    pub fn with_log_types(self, log_types: &'a [&'a str]) -> Self {
        Extension {
            log_types: Some(log_types),
            ..self
        }
    }

    /// Create a new [`Extension`] with specific configuration to buffer logs.
    pub fn with_log_buffering(self, lb: LogBuffering) -> Self {
        Extension {
            log_buffering: Some(lb),
            ..self
        }
    }

    /// Execute the given extension
    pub async fn run(self) -> Result<(), Error> {
        let client = &Client::builder().build()?;

        let extension_id = register(client, self.extension_name, self.events).await?;
        let mut ep = self.events_processor;

        if let Some(mut lp) = self.logs_processor {
            // fixme(david): 
            //   - Spawn task to run processor
            //   - Call Logs API to start receiving vents
        }

        let incoming = async_stream::stream! {
            loop {
                trace!("Waiting for next event (incoming loop)");
                let req = requests::next_event_request(extension_id)?;
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
                extension_id: extension_id.to_string(),
                next: event,
            };

            let res = ep.call(event).await;
            if let Err(error) = res {
                let req = if is_invoke {
                    requests::init_error(extension_id, &error.to_string(), None)?
                } else {
                    requests::exit_error(extension_id, &error.to_string(), None)?
                };

                client.call(req).await?;
                return Err(error.into());
            }
        }
        Ok(())
    }
}

/// A no-op generic processor
pub struct Identity<T> {
    _pd: std::marker::PhantomData<T>,
}

impl<T> Identity<T> {
    fn new() -> Identity<T> {
        Identity {
            _pd: std::marker::PhantomData,
        }
    }
}

impl<T> Service<T> for Identity<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    type Response = ();

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _event: T) -> Self::Future {
        Box::pin(ready(Ok(())))
    }
}

/// Initialize and register the extension in the Extensions API
async fn register<'a>(
    client: &'a Client,
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
) -> Result<&'a str, Error> {
    let name = match extension_name {
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

    let events = events.unwrap_or(&["INVOKE", "SHUTDOWN"]);

    let req = requests::register_request(&name, events)?;
    let res = client.call(req).await?;
    if res.status() != http::StatusCode::OK {
        return Err(ExtensionError::boxed("unable to register the extension"));
    }

    let extension_id = res.headers().get(requests::EXTENSION_ID_HEADER).unwrap().to_str()?;
    Ok("asdf")
}
