#![deny(clippy::all, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]
#![allow(clippy::multiple_crate_versions)]

//! This crate includes a base HTTP client to interact with
//! the AWS Lambda Runtime API.
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};
use http::{
    uri::{PathAndQuery, Scheme},
    Request, Response, Uri,
};
use hyper::body::Incoming;
use hyper_util::client::legacy::connect::HttpConnector;
use std::{convert::TryInto, fmt::Debug, future};

const USER_AGENT_HEADER: &str = "User-Agent";
const DEFAULT_USER_AGENT: &str = concat!("aws-lambda-rust/", env!("CARGO_PKG_VERSION"));
const CUSTOM_USER_AGENT: Option<&str> = option_env!("LAMBDA_RUNTIME_USER_AGENT");

mod error;
pub use error::*;
pub mod body;

#[cfg(feature = "tracing")]
pub mod tracing;

/// API client to interact with the AWS Lambda Runtime API.
#[derive(Debug)]
pub struct Client {
    /// The runtime API URI
    pub base: Uri,
    /// The client that manages the API connections
    pub client: hyper_util::client::legacy::Client<HttpConnector, body::Body>,
}

impl Client {
    /// Create a builder struct to configure the client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            connector: HttpConnector::new(),
            uri: None,
        }
    }
}

impl Client {
    /// Send a given request to the Runtime API.
    /// Use the client's base URI to ensure the API endpoint is correct.
    pub fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>> {
        // NOTE: This method returns a boxed future such that the future has a static lifetime.
        //       Due to limitations around the Rust async implementation as of Mar 2024, this is
        //       required to minimize constraints on the handler passed to [lambda_runtime::run].
        let req = match self.set_origin(req) {
            Ok(req) => req,
            Err(err) => return future::ready(Err(err)).boxed(),
        };
        self.client.request(req).map_err(Into::into).boxed()
    }

    /// Create a new client with a given base URI and HTTP connector.
    fn with(base: Uri, connector: HttpConnector) -> Self {
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .http1_max_buf_size(1024 * 1024)
            .build(connector);
        Self { base, client }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, BoxError> {
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
pub struct ClientBuilder {
    connector: HttpConnector,
    uri: Option<http::Uri>,
}

impl ClientBuilder {
    /// Create a new builder with a given HTTP connector.
    pub fn with_connector(self, connector: HttpConnector) -> ClientBuilder {
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
    pub fn build(self) -> Result<Client, Error> {
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
