#![deny(clippy::all, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]
#![allow(clippy::multiple_crate_versions)]

//! This crate includes a base HTTP client to interact with
//! the AWS Lambda Runtime API.
use http::{uri::PathAndQuery, uri::Scheme, Request, Response, Uri};
use hyper::{
    client::{connect::Connection, HttpConnector},
    Body,
};
use std::{convert::TryInto, fmt::Debug};
use tokio::io::{AsyncRead, AsyncWrite};
use tower_service::Service;

const USER_AGENT_HEADER: &str = "User-Agent";
const DEFAULT_USER_AGENT: &str = concat!("aws-lambda-rust/", env!("CARGO_PKG_VERSION"));
const CUSTOM_USER_AGENT: Option<&str> = option_env!("LAMBDA_RUNTIME_USER_AGENT");

/// Error type that lambdas may result in
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// API client to interact with the AWS Lambda Runtime API.
#[derive(Debug)]
pub struct Client<C = HttpConnector> {
    /// The runtime API URI
    pub base: Uri,
    /// The client that manages the API connections
    pub client: hyper::Client<C>,
}

impl Client {
    /// Create a builder struct to configure the client.
    pub fn builder() -> ClientBuilder<HttpConnector> {
        ClientBuilder {
            connector: HttpConnector::new(),
            uri: None,
        }
    }
}

impl<C> Client<C>
where
    C: hyper::client::connect::Connect + Sync + Send + Clone + 'static,
{
    /// Send a given request to the Runtime API.
    /// Use the client's base URI to ensure the API endpoint is correct.
    pub async fn call(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let req = self.set_origin(req)?;
        let response = self.client.request(req).await?;
        Ok(response)
    }

    /// Create a new client with a given base URI and HTTP connector.
    pub fn with(base: Uri, connector: C) -> Self {
        let client = hyper::Client::builder()
            .http1_max_buf_size(1024 * 1024)
            .build(connector);
        Self { base, client }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Error> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority, base_path) = {
            let scheme = self.base.scheme().unwrap_or(&Scheme::HTTP);
            let authority = self.base.authority().expect("Authority not found");
            let base_path = self.base.path().trim_end_matches('/');
            (scheme, authority, base_path)
        };
        let path = parts.uri.path_and_query().expect("PathAndQuery not found");
        let pq: PathAndQuery = format!("{base_path}{path}").parse().expect("PathAndQuery invalid");

        let uri = Uri::builder()
            .scheme(scheme.as_ref())
            .authority(authority.as_ref())
            .path_and_query(pq)
            .build()
            .map_err(Box::new)?;

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }
}

/// Builder implementation to construct any Runtime API clients.
pub struct ClientBuilder<C: Service<http::Uri> = hyper::client::HttpConnector> {
    connector: C,
    uri: Option<http::Uri>,
}

impl<C> ClientBuilder<C>
where
    C: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
    <C as Service<http::Uri>>::Future: Unpin + Send,
    <C as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    <C as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    /// Create a new builder with a given HTTP connector.
    pub fn with_connector<C2>(self, connector: C2) -> ClientBuilder<C2>
    where
        C2: Service<http::Uri> + Clone + Send + Sync + Unpin + 'static,
        <C2 as Service<http::Uri>>::Future: Unpin + Send,
        <C2 as Service<http::Uri>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        <C2 as Service<http::Uri>>::Response: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    {
        ClientBuilder {
            connector,
            uri: self.uri,
        }
    }

    /// Create a new builder with a given base URI.
    /// Inherits all other attributes from the existent builder.
    pub fn with_endpoint(self, uri: http::Uri) -> Self {
        Self { uri: Some(uri), ..self }
    }

    /// Create the new client to interact with the Runtime API.
    pub fn build(self) -> Result<Client<C>, Error> {
        let uri = match self.uri {
            Some(uri) => uri,
            None => {
                let uri = std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
                uri.try_into().expect("Unable to convert to URL")
            }
        };
        Ok(Client::with(uri, self.connector))
    }
}

/// Create a request builder.
/// This builder uses `aws-lambda-rust/CRATE_VERSION` as
/// the default User-Agent.
/// Configure environment variable `LAMBDA_RUNTIME_USER_AGENT`
/// at compile time to modify User-Agent value.
pub fn build_request() -> http::request::Builder {
    const USER_AGENT: &str = match CUSTOM_USER_AGENT {
        Some(value) => value,
        None => DEFAULT_USER_AGENT,
    };
    http::Request::builder().header(USER_AGENT_HEADER, USER_AGENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_origin() {
        let base = "http://localhost:9001";
        let client = Client::builder().with_endpoint(base.parse().unwrap()).build().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = client.set_origin(req).unwrap();
        assert_eq!(
            "http://localhost:9001/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );
    }

    #[test]
    fn test_set_origin_with_base_path() {
        let base = "http://localhost:9001/foo";
        let client = Client::builder().with_endpoint(base.parse().unwrap()).build().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = client.set_origin(req).unwrap();
        assert_eq!(
            "http://localhost:9001/foo/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );

        let base = "http://localhost:9001/foo/";
        let client = Client::builder().with_endpoint(base.parse().unwrap()).build().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = client.set_origin(req).unwrap();
        assert_eq!(
            "http://localhost:9001/foo/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );
    }
}
