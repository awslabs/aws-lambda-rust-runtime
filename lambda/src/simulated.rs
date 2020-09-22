use http::Uri;
use hyper::client::connect::Connection;
use std::{
    cmp::min,
    collections::VecDeque,
    future::Future,
    io::Result as IoResult,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tower_service::Service;

/// Creates a pair of `AsyncRead`/`AsyncWrite` data streams, where the write end of each member of the pair
/// is the read end of the other member of the pair.  This allows us to emulate the behavior of a TcpStream
/// but in-memory, deterministically, and with full control over failure injection.
pub(crate) fn chan() -> (SimStream, SimStream) {
    // Set up two reference-counted, lock-guarded byte VecDeques, one for each direction of the
    // connection
    let one = Arc::new(Mutex::new(BufferState::new()));
    let two = Arc::new(Mutex::new(BufferState::new()));

    // Use buf1 for the read-side of left, use buf2 for the write-side of left
    let left = SimStream {
        read: ReadHalf { buffer: one.clone() },
        write: WriteHalf { buffer: two.clone() },
    };

    // Now swap the buffers for right
    let right = SimStream {
        read: ReadHalf { buffer: two },
        write: WriteHalf { buffer: one },
    };

    (left, right)
}

#[derive(Clone)]
pub struct Connector {
    pub inner: SimStream,
}

impl Service<Uri> for Connector {
    type Response = SimStream;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Uri) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move { Ok(inner) })
    }
}

impl Connection for SimStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

/// A struct that implements AsyncRead + AsyncWrite (similarly to TcpStream) using in-memory
/// bytes only.  Unfortunately tokio does not provide an operation that is the opposite of
/// `tokio::io::split`, as that would negate the need for this struct.
// TODO: Implement the ability to explicitly close a connection
#[derive(Debug, Clone)]
pub struct SimStream {
    read: ReadHalf,
    write: WriteHalf,
}

/// Delegates to the underlying `write` member's methods
impl AsyncWrite for SimStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.write).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.write).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.write).poll_shutdown(cx)
    }
}

/// Delegates to the underlying `read` member's methods
impl AsyncRead for SimStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.read).poll_read(cx, buf)
    }
}

/// A buffer for use with ReadHalf/WriteHalf that allows bytes to be written at one end of a
/// dequeue and read from the other end.  If a `read_waker` is provided, the BufferState will call
/// `wake()` when there is new data to be read.
#[derive(Debug, Clone)]
pub struct BufferState {
    buffer: VecDeque<u8>,
    read_waker: Option<Waker>,
}

impl BufferState {
    /// Creates a new `BufferState`.
    fn new() -> Self {
        BufferState {
            buffer: VecDeque::new(),
            read_waker: None,
        }
    }
    /// Writes data to the front of the deque byte buffer
    fn write(&mut self, buf: &[u8]) {
        for b in buf {
            self.buffer.push_front(*b)
        }

        // If somebody is waiting on this data, wake them up.
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
    }

    /// Read data from the end of the deque byte buffer
    fn read(&mut self, to_buf: &mut [u8]) -> usize {
        // Read no more bytes than we have available, and no more bytes than we were asked for
        let bytes_to_read = min(to_buf.len(), self.buffer.len());
        for i in 0..bytes_to_read {
            to_buf[i] = self.buffer.pop_back().unwrap();
        }

        bytes_to_read
    }
}

/// An AsyncWrite implementation that uses a VecDeque of bytes as a buffer.  The WriteHalf will
/// add new bytes to the front of the deque using push_front.
///
/// Intended for use with ReadHalf to read from the VecDeque
#[derive(Debug, Clone)]
pub struct WriteHalf {
    buffer: Arc<Mutex<BufferState>>,
}

impl AsyncWrite for WriteHalf {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        // Acquire the lock for the buffer
        let mut write_to = self
            .buffer
            .lock()
            .expect("Lock was poisoned when acquiring buffer lock for WriteHalf");

        // write the bytes
        write_to.write(buf);

        // This operation completes immediately
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug, Clone)]
pub struct ReadHalf {
    buffer: Arc<Mutex<BufferState>>,
}

impl AsyncRead for ReadHalf {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<IoResult<usize>> {
        // Acquire the lock for the buffer
        let mut read_from = self
            .buffer
            .lock()
            .expect("Lock was poisoned when acquiring buffer lock for ReadHalf");

        let bytes_read = read_from.read(buf);

        // Returning Poll::Ready(Ok(0)) would indicate that there is nothing more to read, which
        // means that someone trying to read from a VecDeque that hasn't been written to yet
        // would get an Eof error (as I learned the hard way).  Instead we should return Poll:Pending
        // to indicate that there could be more to read in the future.
        if (bytes_read) == 0 {
            read_from.read_waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            //read_from.read_waker = Some(cx.waker().clone());
            Poll::Ready(Ok(bytes_read))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::chan;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn ends_should_talk_to_each_other() {
        let (mut client, mut server) = chan();
        // Write ping to the side 1
        client.write_all(b"Ping").await.expect("Write should succeed");

        // Verify we can read it from side 2
        let mut read_on_server = [0_u8; 4];
        server
            .read_exact(&mut read_on_server)
            .await
            .expect("Read should succeed");
        assert_eq!(&read_on_server, b"Ping");

        // Write "Pong" to side 2
        server.write_all(b"Pong").await.expect("Write should succeed");

        // Verify we can read it from side 1
        let mut read_on_client = [0_u8; 4];
        client
            .read_exact(&mut read_on_client)
            .await
            .expect("Read should succeed");
        assert_eq!(&read_on_client, b"Pong");
    }
}
