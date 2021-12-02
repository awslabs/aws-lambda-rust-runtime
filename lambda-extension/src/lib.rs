// #![deny(clippy::all, clippy::cargo)]
// #![warn(missing_docs,? nonstandard_style, rust_2018_idioms)]

use hyper::client::{connect::Connection, HttpConnector};
use lambda_runtime_api_client::Client;
use serde::Deserialize;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tower_service::Service;
use tracing::trace;

pub mod requests;

pub type Error = lambda_runtime_api_client::Error;
pub type ExtensionId = String;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracing {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeEvent {
    deadline_ms: u64,
    request_id: String,
    invoked_function_arn: String,
    tracing: Tracing,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownEvent {
    shutdown_reason: String,
    deadline_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
pub enum NextEvent {
    Invoke(InvokeEvent),
    Shutdown(ShutdownEvent),
}

impl NextEvent {
    fn is_invoke(&self) -> bool {
        match self {
            NextEvent::Invoke(_) => true,
            _ => false,
        }
    }
}

/// A trait describing an asynchronous extension.
pub trait Extension {
    /// Response of this Extension.
    type Fut: Future<Output = Result<(), Error>>;
    /// Handle the incoming event.
    fn call(&self, extension_id: ExtensionId, event: NextEvent) -> Self::Fut;
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
    F: Fn(ExtensionId, NextEvent) -> Fut,
    Fut: Future<Output = Result<(), Error>>,
{
    type Fut = Fut;
    fn call(&self, extension_id: ExtensionId, event: NextEvent) -> Self::Fut {
        (self.f)(extension_id, event)
    }
}

pub struct Runtime<C: Service<http::Uri> = HttpConnector> {
    extension_id: ExtensionId,
    client: Client<C>,
}

impl Runtime {
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
    pub async fn run(&self, extension: impl Extension) -> Result<(), Error> {
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

            let res = extension.call(self.extension_id.clone(), event).await;
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

#[derive(Default)]
pub struct RuntimeBuilder<'a> {
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
}

impl<'a> RuntimeBuilder<'a> {
    pub fn with_extension_name(self, extension_name: &'a str) -> Self {
        RuntimeBuilder {
            extension_name: Some(extension_name),
            ..self
        }
    }

    pub fn with_events(self, events: &'a [&'a str]) -> Self {
        RuntimeBuilder {
            events: Some(events),
            ..self
        }
    }

    pub async fn register(&self) -> Result<Runtime, Error> {
        let name = match self.extension_name {
            Some(name) => name.into(),
            None => {
                let args: Vec<String> = std::env::args().collect();
                args[0].clone()
            }
        };

        let events = match self.events {
            Some(events) => events,
            None => &["INVOKE", "SHUTDOWN"],
        };

        let client = Client::builder().build()?;

        let req = requests::register_request(&name, events)?;
        let res = client.call(req).await?;
        // ensure!(res.status() == http::StatusCode::OK, "Unable to register extension",);

        let extension_id = res.headers().get(requests::EXTENSION_ID_HEADER).unwrap().to_str()?;
        Ok(Runtime {
            extension_id: extension_id.into(),
            client: client,
        })
    }
}

pub async fn run<Ex>(extension: Ex) -> Result<(), Error>
where
    Ex: Extension,
{
    Runtime::builder().register().await?.run(extension).await
}
