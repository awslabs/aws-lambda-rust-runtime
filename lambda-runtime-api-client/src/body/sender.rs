//! Body::channel utilities. Extracted from Hyper under MIT license.
//! https://github.com/hyperium/hyper/blob/master/LICENSE

use crate::Error;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_channel::{mpsc, oneshot};
use http::HeaderMap;

use super::watch;

type BodySender = mpsc::Sender<Result<Bytes, Error>>;
type TrailersSender = oneshot::Sender<HeaderMap>;

pub(crate) const WANT_PENDING: usize = 1;
pub(crate) const WANT_READY: usize = 2;

/// A sender half created through [`Body::channel()`].
///
/// Useful when wanting to stream chunks from another thread.
///
/// ## Body Closing
///
/// Note that the request body will always be closed normally when the sender is dropped (meaning
/// that the empty terminating chunk will be sent to the remote). If you desire to close the
/// connection with an incomplete response (e.g. in the case of an error during asynchronous
/// processing), call the [`Sender::abort()`] method to abort the body in an abnormal fashion.
///
/// [`Body::channel()`]: struct.Body.html#method.channel
/// [`Sender::abort()`]: struct.Sender.html#method.abort
#[must_use = "Sender does nothing unless sent on"]
pub struct Sender {
    pub(crate) want_rx: watch::Receiver,
    pub(crate) data_tx: BodySender,
    pub(crate) trailers_tx: Option<TrailersSender>,
}

impl Sender {
    /// Check to see if this `Sender` can send more data.
    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        // Check if the receiver end has tried polling for the body yet
        ready!(self.poll_want(cx)?);
        self.data_tx
            .poll_ready(cx)
            .map_err(|_| Error::new(SenderError::ChannelClosed))
    }

    fn poll_want(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.want_rx.load(cx) {
            WANT_READY => Poll::Ready(Ok(())),
            WANT_PENDING => Poll::Pending,
            watch::CLOSED => Poll::Ready(Err(Error::new(SenderError::ChannelClosed))),
            unexpected => unreachable!("want_rx value: {}", unexpected),
        }
    }

    async fn ready(&mut self) -> Result<(), Error> {
        futures_util::future::poll_fn(|cx| self.poll_ready(cx)).await
    }

    /// Send data on data channel when it is ready.
    #[allow(unused)]
    pub async fn send_data(&mut self, chunk: Bytes) -> Result<(), Error> {
        self.ready().await?;
        self.data_tx
            .try_send(Ok(chunk))
            .map_err(|_| Error::new(SenderError::ChannelClosed))
    }

    /// Send trailers on trailers channel.
    #[allow(unused)]
    pub async fn send_trailers(&mut self, trailers: HeaderMap) -> Result<(), Error> {
        let tx = match self.trailers_tx.take() {
            Some(tx) => tx,
            None => return Err(Error::new(SenderError::ChannelClosed)),
        };
        tx.send(trailers).map_err(|_| Error::new(SenderError::ChannelClosed))
    }

    /// Try to send data on this channel.
    ///
    /// # Errors
    ///
    /// Returns `Err(Bytes)` if the channel could not (currently) accept
    /// another `Bytes`.
    ///
    /// # Note
    ///
    /// This is mostly useful for when trying to send from some other thread
    /// that doesn't have an async context. If in an async context, prefer
    /// `send_data()` instead.
    pub fn try_send_data(&mut self, chunk: Bytes) -> Result<(), Bytes> {
        self.data_tx
            .try_send(Ok(chunk))
            .map_err(|err| err.into_inner().expect("just sent Ok"))
    }

    /// Send a `SenderError::BodyWriteAborted` error and terminate the stream.
    #[allow(unused)]
    pub fn abort(mut self) {
        self.send_error(Error::new(SenderError::BodyWriteAborted));
    }

    /// Terminate the stream with an error.
    pub fn send_error(&mut self, err: Error) {
        let _ = self
            .data_tx
            // clone so the send works even if buffer is full
            .clone()
            .try_send(Err(err));
    }
}

#[derive(Debug)]
enum SenderError {
    ChannelClosed,
    BodyWriteAborted,
}

impl SenderError {
    fn description(&self) -> &str {
        match self {
            SenderError::BodyWriteAborted => "user body write aborted",
            SenderError::ChannelClosed => "channel closed",
        }
    }
}

impl std::fmt::Display for SenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}
impl std::error::Error for SenderError {}
