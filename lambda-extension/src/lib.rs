#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! This module includes utilities to create Lambda Runtime Extensions.
//!
//! Create a type that conforms to the [`Extension`] trait. This type can then be passed
//! to the the `lambda_extension::run` function, which launches and runs the Lambda runtime extension.
use hyper::client::{connect::Connection, HttpConnector};
use lambda_runtime_api_client::Client;
use serde::Deserialize;
use std::{future::Future, path::PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tower_service::Service;
use tracing::trace;

/// Include several request builders to interact with the Extension API.
pub mod requests;

/// Error type that extensions may result in
pub type Error = lambda_runtime_api_client::Error;

/// Simple error that encapsulates human readable descriptions
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionError {
    err: String,
}

impl ExtensionError {
    fn boxed<T: Into<String>>(str: T) -> Box<ExtensionError> {
        Box::new(ExtensionError { err: str.into() })
    }
}

impl std::fmt::Display for ExtensionError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::error::Error for ExtensionError {}

/// Request tracing information
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracing {
    /// The type of tracing exposed to the extension
    pub r#type: String,
    /// The span value
    pub value: String,
}

/// Event received when there is a new Lambda invocation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeEvent {
    /// The time that the function times out
    pub deadline_ms: u64,
    /// The ID assigned to the Lambda request
    pub request_id: String,
    /// The function's Amazon Resource Name
    pub invoked_function_arn: String,
    /// The request tracing information
    pub tracing: Tracing,
}

/// Event received when a Lambda function shuts down.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownEvent {
    /// The reason why the function terminates
    /// It can be SPINDOWN, TIMEOUT, or FAILURE
    pub shutdown_reason: String,
    /// The time that the function times out
    pub deadline_ms: u64,
}

/// Event that the extension receives in
/// either the INVOKE or SHUTDOWN phase
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
pub enum NextEvent {
    /// Payload when the event happens in the INVOKE phase
    Invoke(InvokeEvent),
    /// Payload when the event happens in the SHUTDOWN phase
    Shutdown(ShutdownEvent),
}

impl NextEvent {
    fn is_invoke(&self) -> bool {
        matches!(self, NextEvent::Invoke(_))
    }
}

/// Wrapper with information about the next
/// event that the Lambda Runtime is going to process
pub struct LambdaEvent {
    /// ID assigned to this extension by the Lambda Runtime
    pub extension_id: String,
    /// Next incoming event
    pub next: NextEvent,
}

/// A trait describing an asynchronous extension.
pub trait Extension {
    /// Response of this Extension.
    type Fut: Future<Output = Result<(), Error>>;
    /// Handle the incoming event.
    fn call(&mut self, event: LambdaEvent) -> Self::Fut;
}

/// Returns a new [`ExtensionFn`] with the given closure.
///
/// [`ExtensionFn`]: struct.ExtensionFn.html
pub fn extension_fn<F>(f: F) -> ExtensionFn<F> {
    ExtensionFn { f }
}

/// An [`Extension`] implemented by a closure.
///
/// [`Extension`]: trait.Extension.html
#[derive(Clone, Debug)]
pub struct ExtensionFn<F> {
    f: F,
}

impl<F, Fut> Extension for ExtensionFn<F>
where
    F: Fn(LambdaEvent) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    type Fut = Fut;
    fn call(&mut self, event: LambdaEvent) -> Self::Fut {
        (self.f)(event)
    }
}

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
    <C as Service<http::Uri>>::Future: Unpin + Send,
    <C as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    <C as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    /// Execute the given extension.
    /// Register the extension with the Extensions API and wait for incoming events.
    pub async fn run(&self, mut extension: impl Extension) -> Result<(), Error> {
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
                return Err(error);
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

/// Execute the given extension
pub async fn run<Ex>(extension: Ex) -> Result<(), Error>
where
    Ex: Extension,
{
    Runtime::builder().register().await?.run(extension).await
}
