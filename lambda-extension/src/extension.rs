use crate::{logs::*, requests, Error, ExtensionError, LambdaEvent, NextEvent};
use hyper::{server::conn::AddrStream, service::make_service_fn, Server};
use lambda_runtime_api_client::Client;
use std::{fmt, future::ready, future::Future, net::SocketAddr, path::PathBuf, pin::Pin};
use tokio_stream::StreamExt;
use tower::{MakeService, Service};
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

impl<'a> Extension<'a, Identity<LambdaEvent>, MakeIdentity<LambdaLog>> {
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

impl<'a> Default for Extension<'a, Identity<LambdaEvent>, MakeIdentity<LambdaLog>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E, L> Extension<'a, E, L>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,

    // Fixme: 'static bound might be too restrictive
    L: MakeService<(), LambdaLog, Response = ()> + Send + Sync + 'static,
    L::Service: Service<LambdaLog, Response = ()> + Send + Sync,
    <L::Service as Service<LambdaLog>>::Future: Send + 'a,
    L::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    L::MakeError: Into<Box<dyn std::error::Error + Send + Sync>>,
    L::Future: Send,
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
        let extension_id = extension_id.to_str()?;
        let mut ep = self.events_processor;

        if let Some(mut log_processor) = self.logs_processor {
            // Spawn task to run processor
            let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
            // let make_service = LogAdapter::new(log_processor);
            let make_service = make_service_fn(move |_socket: &AddrStream| {
                let service = log_processor.make_service(());
                async move { Ok::<_, L::MakeError>(LogAdapter::new(service.await?)) }
            });
            let server = Server::bind(&addr).serve(make_service);
            tokio::spawn(async move { server.await });

            // Call Logs API to start receiving events
            let req = requests::subscribe_logs_request(extension_id, self.log_types, self.log_buffering)?;
            let res = client.call(req).await?;
            if res.status() != http::StatusCode::OK {
                return Err(ExtensionError::boxed("unable to initialize the logs api"));
            }
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

            let event = LambdaEvent::new(extension_id, event);

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
#[derive(Clone)]
pub struct Identity<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Identity<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Service<T> for Identity<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;
    type Response = ();

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _event: T) -> Self::Future {
        Box::pin(ready(Ok(())))
    }
}

/// Service factory to generate no-op generic processors
#[derive(Clone)]
pub struct MakeIdentity<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Service<()> for MakeIdentity<T>
where
    T: Send + Sync + 'static,
{
    type Error = Error;
    type Response = Identity<T>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        Box::pin(ready(Ok(Identity::new())))
    }
}

/// Initialize and register the extension in the Extensions API
async fn register<'a>(
    client: &'a Client,
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
) -> Result<http::HeaderValue, Error> {
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

    let header = res
        .headers()
        .get(requests::EXTENSION_ID_HEADER)
        .ok_or_else(|| ExtensionError::boxed("missing extension id header"))
        .map_err(|e| ExtensionError::boxed(e.to_string()))?;
    Ok(header.clone())
}
