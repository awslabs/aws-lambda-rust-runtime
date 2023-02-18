use http::Uri;
use hyper::client::connect::Connection;
use std::{
    collections::HashMap,
    future::Future,
    io::Result as IoResult,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, DuplexStream, ReadBuf};

use crate::Error;

#[derive(Clone)]
pub struct Connector {
    inner: Arc<Mutex<HashMap<Uri, DuplexStreamWrapper>>>,
}

pub struct DuplexStreamWrapper(DuplexStream);

impl DuplexStreamWrapper {
    pub(crate) fn new(stream: DuplexStream) -> DuplexStreamWrapper {
        DuplexStreamWrapper(stream)
    }
}

impl Connector {
    pub fn new() -> Self {
        #[allow(clippy::mutable_key_type)]
        let map = HashMap::new();
        Connector {
            inner: Arc::new(Mutex::new(map)),
        }
    }

    pub fn insert(&self, uri: Uri, stream: DuplexStreamWrapper) -> Result<(), Error> {
        match self.inner.lock() {
            Ok(mut map) => {
                map.insert(uri, stream);
                Ok(())
            }
            Err(_) => Err("mutex was poisoned".into()),
        }
    }

    pub fn with(uri: Uri, stream: DuplexStreamWrapper) -> Result<Self, Error> {
        let connector = Connector::new();
        match connector.insert(uri, stream) {
            Ok(_) => Ok(connector),
            Err(e) => Err(e),
        }
    }
}

impl hyper::service::Service<Uri> for Connector {
    type Response = DuplexStreamWrapper;
    type Error = crate::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let res = match self.inner.lock() {
            Ok(mut map) if map.contains_key(&uri) => Ok(map.remove(&uri).unwrap()),
            Ok(_) => Err(format!("Uri {uri} is not in map").into()),
            Err(_) => Err("mutex was poisoned".into()),
        };
        Box::pin(async move { res })
    }
}

impl Connection for DuplexStreamWrapper {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

impl AsyncRead for DuplexStreamWrapper {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for DuplexStreamWrapper {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
