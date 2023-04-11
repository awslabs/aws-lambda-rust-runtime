use std::{
    convert::Infallible, fmt, future::ready, future::Future, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc,
};

use hyper::{server::conn::AddrStream, Server};
use lambda_runtime_api_client::Client;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tower::{service_fn, MakeService, Service, ServiceExt};
use tracing::{error, trace};

use crate::{
    logs::*,
    requests::{self, Api},
    telemetry_wrapper, Error, ExtensionError, LambdaEvent, LambdaTelemetry, NextEvent,
};

const DEFAULT_LOG_PORT_NUMBER: u16 = 9002;
const DEFAULT_TELEMETRY_PORT_NUMBER: u16 = 9003;

/// An Extension that runs event, log and telemetry processors
pub struct Extension<'a, E, L, T> {
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
    events_processor: E,
    log_types: Option<&'a [&'a str]>,
    logs_processor: Option<L>,
    log_buffering: Option<LogBuffering>,
    log_port_number: u16,
    telemetry_types: Option<&'a [&'a str]>,
    telemetry_processor: Option<T>,
    telemetry_buffering: Option<LogBuffering>,
    telemetry_port_number: u16,
}

impl<'a> Extension<'a, Identity<LambdaEvent>, MakeIdentity<Vec<LambdaLog>>, MakeIdentity<Vec<LambdaTelemetry>>> {
    /// Create a new base [`Extension`] with a no-op events processor
    pub fn new() -> Self {
        Extension {
            extension_name: None,
            events: None,
            events_processor: Identity::new(),
            log_types: None,
            log_buffering: None,
            logs_processor: None,
            log_port_number: DEFAULT_LOG_PORT_NUMBER,
            telemetry_types: None,
            telemetry_buffering: None,
            telemetry_processor: None,
            telemetry_port_number: DEFAULT_TELEMETRY_PORT_NUMBER,
        }
    }
}

impl<'a> Default
    for Extension<'a, Identity<LambdaEvent>, MakeIdentity<Vec<LambdaLog>>, MakeIdentity<Vec<LambdaTelemetry>>>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E, L, T> Extension<'a, E, L, T>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display + fmt::Debug,

    // Fixme: 'static bound might be too restrictive
    L: MakeService<(), Vec<LambdaLog>, Response = ()> + Send + Sync + 'static,
    L::Service: Service<Vec<LambdaLog>, Response = ()> + Send + Sync,
    <L::Service as Service<Vec<LambdaLog>>>::Future: Send + 'a,
    L::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Debug,
    L::MakeError: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Debug,
    L::Future: Send,

    // Fixme: 'static bound might be too restrictive
    T: MakeService<(), Vec<LambdaTelemetry>, Response = ()> + Send + Sync + 'static,
    T::Service: Service<Vec<LambdaTelemetry>, Response = ()> + Send + Sync,
    <T::Service as Service<Vec<LambdaTelemetry>>>::Future: Send + 'a,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Debug,
    T::MakeError: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Debug,
    T::Future: Send,
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
    pub fn with_events_processor<N>(self, ep: N) -> Extension<'a, N, L, T>
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
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_processor: self.telemetry_processor,
            telemetry_port_number: self.telemetry_port_number,
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda logs.
    pub fn with_logs_processor<N, NS>(self, lp: N) -> Extension<'a, E, N, T>
    where
        N: Service<()>,
        N::Future: Future<Output = Result<NS, N::Error>>,
        N::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
    {
        Extension {
            logs_processor: Some(lp),
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_processor: self.telemetry_processor,
            telemetry_port_number: self.telemetry_port_number,
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

    /// Create a new [`Extension`] with a different port number to listen to logs.
    pub fn with_log_port_number(self, port_number: u16) -> Self {
        Extension {
            log_port_number: port_number,
            ..self
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda telemetry data.
    pub fn with_telemetry_processor<N, NS>(self, lp: N) -> Extension<'a, E, L, N>
    where
        N: Service<()>,
        N::Future: Future<Output = Result<NS, N::Error>>,
        N::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
    {
        Extension {
            telemetry_processor: Some(lp),
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            logs_processor: self.logs_processor,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_port_number: self.telemetry_port_number,
        }
    }

    /// Create a new [`Extension`] with a list of telemetry types to subscribe.
    /// The only accepted telemetry types are `function`, `platform`, and `extension`.
    pub fn with_telemetry_types(self, telemetry_types: &'a [&'a str]) -> Self {
        Extension {
            telemetry_types: Some(telemetry_types),
            ..self
        }
    }

    /// Create a new [`Extension`] with specific configuration to buffer telemetry.
    pub fn with_telemetry_buffering(self, lb: LogBuffering) -> Self {
        Extension {
            telemetry_buffering: Some(lb),
            ..self
        }
    }

    /// Create a new [`Extension`] with a different port number to listen to telemetry.
    pub fn with_telemetry_port_number(self, port_number: u16) -> Self {
        Extension {
            telemetry_port_number: port_number,
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
            trace!("Log processor found");
            // Spawn task to run processor
            let addr = SocketAddr::from(([0, 0, 0, 0], self.log_port_number));
            let make_service = service_fn(move |_socket: &AddrStream| {
                trace!("Creating new log processor Service");
                let service = log_processor.make_service(());
                async move {
                    let service = Arc::new(Mutex::new(service.await?));
                    Ok::<_, L::MakeError>(service_fn(move |req| log_wrapper(service.clone(), req)))
                }
            });
            let server = Server::bind(&addr).serve(make_service);
            tokio::spawn(async move {
                if let Err(e) = server.await {
                    error!("Error while running log processor: {}", e);
                }
            });
            trace!("Log processor started");

            // Call Logs API to start receiving events
            let req = requests::subscribe_request(
                Api::LogsApi,
                extension_id,
                self.log_types,
                self.log_buffering,
                self.log_port_number,
            )?;
            let res = client.call(req).await?;
            if res.status() != http::StatusCode::OK {
                let err = format!("unable to initialize the logs api: {}", res.status());
                return Err(ExtensionError::boxed(err));
            }
            trace!("Registered extension with Logs API");
        }

        if let Some(mut telemetry_processor) = self.telemetry_processor {
            trace!("Telemetry processor found");
            // Spawn task to run processor
            let addr = SocketAddr::from(([0, 0, 0, 0], self.telemetry_port_number));
            let make_service = service_fn(move |_socket: &AddrStream| {
                trace!("Creating new telemetry processor Service");
                let service = telemetry_processor.make_service(());
                async move {
                    let service = Arc::new(Mutex::new(service.await?));
                    Ok::<_, T::MakeError>(service_fn(move |req| telemetry_wrapper(service.clone(), req)))
                }
            });
            let server = Server::bind(&addr).serve(make_service);
            tokio::spawn(async move {
                if let Err(e) = server.await {
                    error!("Error while running telemetry processor: {}", e);
                }
            });
            trace!("Telemetry processor started");

            // Call Telemetry API to start receiving events
            let req = requests::subscribe_request(
                Api::TelemetryApi,
                extension_id,
                self.telemetry_types,
                self.telemetry_buffering,
                self.telemetry_port_number,
            )?;
            let res = client.call(req).await?;
            if res.status() != http::StatusCode::OK {
                let err = format!("unable to initialize the telemetry api: {}", res.status());
                return Err(ExtensionError::boxed(err));
            }
            trace!("Registered extension with Telemetry API");
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

            let event = LambdaEvent::new(event);

            let ep = match ep.ready().await {
                Ok(ep) => ep,
                Err(err) => {
                    println!("Inner service is not ready: {err:?}");
                    let req = if is_invoke {
                        requests::init_error(extension_id, &err.to_string(), None)?
                    } else {
                        requests::exit_error(extension_id, &err.to_string(), None)?
                    };

                    client.call(req).await?;
                    return Err(err.into());
                }
            };

            let res = ep.call(event).await;
            if let Err(err) = res {
                println!("{err:?}");
                let req = if is_invoke {
                    requests::init_error(extension_id, &err.to_string(), None)?
                } else {
                    requests::exit_error(extension_id, &err.to_string(), None)?
                };

                client.call(req).await?;
                return Err(err.into());
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
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
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
    type Error = Infallible;
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
        let err = format!("unable to register the extension: {}", res.status());
        return Err(ExtensionError::boxed(err));
    }

    let header = res
        .headers()
        .get(requests::EXTENSION_ID_HEADER)
        .ok_or_else(|| ExtensionError::boxed("missing extension id header"))
        .map_err(|e| ExtensionError::boxed(e.to_string()))?;
    Ok(header.clone())
}
