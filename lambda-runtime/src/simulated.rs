use http::Uri;
use hyper::rt::{Read, Write};
use hyper_util::client::legacy::connect::{Connected, Connection};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::io::DuplexStream;

use crate::Error;

#[derive(Clone)]
pub struct Connector {
    inner: Arc<Mutex<HashMap<Uri, DuplexStreamWrapper>>>,
}
use pin_project_lite::pin_project;

pin_project! {
pub struct DuplexStreamWrapper {
    #[pin]
    inner: DuplexStream,
}
}

impl DuplexStreamWrapper {
    pub(crate) fn new(inner: DuplexStream) -> DuplexStreamWrapper {
        DuplexStreamWrapper { inner }
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

impl tower::Service<Uri> for Connector {
    type Response = DuplexStreamWrapper;
    type Error = crate::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, uri: Uri) -> Self::Future {
        let res = match self.inner.lock() {
            Ok(mut map) if map.contains_key(&uri) => Ok(map.remove(&uri).unwrap()),
            Ok(_) => Err(format!("Uri {uri} is not in map").into()),
            Err(_) => Err("mutex was poisoned".into()),
        };
        Box::pin(async move { res })
    }

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Connection for DuplexStreamWrapper {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl Read for DuplexStreamWrapper {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let n = unsafe {
            let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(self.project().inner, cx, &mut tbuf) {
                Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }
        Poll::Ready(Ok(()))
    }
}

impl Write for DuplexStreamWrapper {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        tokio::io::AsyncWrite::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write_vectored(self.project().inner, cx, bufs)
    }
}
