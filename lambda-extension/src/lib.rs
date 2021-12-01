// #![deny(clippy::all, clippy::cargo)]
// #![warn(missing_docs,? nonstandard_style, rust_2018_idioms)]

use async_trait::async_trait;
use hyper::client::{connect::Connection, HttpConnector};
use lambda_runtime_api_client::Client;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::{StreamExt};
use tower_service::Service;
use tracing::trace;

pub mod requests;

pub type Error = lambda_runtime_api_client::Error;

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

/// A trait describing an asynchronous extension.
#[async_trait]
pub trait Extension {
    async fn on_invoke(&self, extension_id: &str, event: InvokeEvent) -> Result<(), Error>;
    async fn on_shutdown(&self, extension_id: &str, event: ShutdownEvent) -> Result<(), Error>;
}

struct Runtime<'a, C: Service<http::Uri> = HttpConnector> {
    extension_id: &'a str,
    client: Client<C>,
}

impl<'a, C> Runtime<'a, C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    <C as Service<http::Uri>>::Future: Unpin + Send,
    <C as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    <C as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn run(&self, extension: impl Extension) -> Result<(), Error> {
        let client = &self.client;
        let extension_id = self.extension_id;

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

            match event {
                NextEvent::Invoke(event) => {
                    extension.on_invoke(extension_id, event).await?;
                }
                NextEvent::Shutdown(event) => {
                    extension.on_shutdown(extension_id, event).await?;
                }
            };
        }

        Ok(())
    }
}

async fn register<C>(client: &Client<C>, extension_name: &str) -> Result<String, Error>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    <C as Service<http::Uri>>::Future: Unpin + Send,
    <C as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    <C as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    let req = requests::register_request(extension_name)?;
    let res = client.call(req).await?;
    // ensure!(res.status() == http::StatusCode::OK, "Unable to register extension",);

    let ext_id = res.headers().get(requests::EXTENSION_ID_HEADER).unwrap().to_str()?;
    Ok(ext_id.into())
}

pub async fn run(extension: impl Extension) -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    let client = Client::builder().build().expect("Unable to create a runtime client");
    let extension_id = register(&client, &args[0]).await?;
    let runtime = Runtime {
        extension_id: &extension_id,
        client,
    };

    runtime.run(extension).await
}
