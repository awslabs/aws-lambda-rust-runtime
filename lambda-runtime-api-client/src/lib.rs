use http::{uri::Scheme, Request, Response, Uri};
use hyper::client::{connect::Connection, HttpConnector};
use hyper::Body;
use std::fmt::Debug;
use tokio::io::{AsyncRead, AsyncWrite};
use tower_service::Service;

const USER_AGENT_HEADER: &str = "User-Agent";
const USER_AGENT: &str = concat!("aws-lambda-rust/", env!("CARGO_PKG_VERSION"));

/// Error type that lambdas may result in
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct Client<C = HttpConnector> {
    pub base: Uri,
    pub client: hyper::Client<C>,
}

impl Client {
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
    pub async fn call(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let req = self.set_origin(req)?;
        let response = self.client.request(req).await?;
        Ok(response)
    }

    pub fn with(base: Uri, connector: C) -> Self {
        let client = hyper::Client::builder().build(connector);
        Self { base, client }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Error> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self.base.scheme().unwrap_or(&Scheme::HTTP);
            let authority = self.base.authority().expect("Authority not found");
            (scheme, authority)
        };
        let path = parts.uri.path_and_query().expect("PathAndQuery not found");

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build();

        match uri {
            Ok(u) => {
                parts.uri = u;
                Ok(Request::from_parts(parts, body))
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

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

    pub fn with_endpoint(self, uri: http::Uri) -> Self {
        Self { uri: Some(uri), ..self }
    }

    pub fn build(self) -> Result<Client<C>, Error> {
        let uri = match self.uri {
            Some(uri) => uri,
            None => {
                let uri = std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
                uri.clone().try_into().expect("Unable to convert to URL")
            }
        };
        Ok(Client::with(uri, self.connector))
    }
}

pub fn build_request() -> http::request::Builder {
    http::Request::builder().header(USER_AGENT_HEADER, USER_AGENT)
}
