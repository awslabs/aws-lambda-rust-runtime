//! Body::channel utilities. Extracted from Hyper under MIT license.
//! https://github.com/hyperium/hyper/blob/master/LICENSE

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use crate::body::{sender, watch};
use bytes::Bytes;
use futures_channel::mpsc;
use futures_channel::oneshot;
use futures_util::{stream::FusedStream, Future, Stream};
use http::HeaderMap;
use http_body::Body;
use http_body::Frame;
use http_body::SizeHint;
pub use sender::Sender;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct DecodedLength(u64);

impl DecodedLength {
    pub(crate) const CLOSE_DELIMITED: DecodedLength = DecodedLength(::std::u64::MAX);
    pub(crate) const CHUNKED: DecodedLength = DecodedLength(::std::u64::MAX - 1);
    pub(crate) const ZERO: DecodedLength = DecodedLength(0);

    pub(crate) fn sub_if(&mut self, amt: u64) {
        match *self {
            DecodedLength::CHUNKED | DecodedLength::CLOSE_DELIMITED => (),
            DecodedLength(ref mut known) => {
                *known -= amt;
            }
        }
    }

    /// Converts to an Option<u64> representing a Known or Unknown length.
    pub(crate) fn into_opt(self) -> Option<u64> {
        match self {
            DecodedLength::CHUNKED | DecodedLength::CLOSE_DELIMITED => None,
            DecodedLength(known) => Some(known),
        }
    }
}

pub struct ChannelBody {
    content_length: DecodedLength,
    want_tx: watch::Sender,
    data_rx: mpsc::Receiver<Result<Bytes, crate::Error>>,
    trailers_rx: oneshot::Receiver<HeaderMap>,
}

pub fn channel() -> (Sender, ChannelBody) {
    let (data_tx, data_rx) = mpsc::channel(0);
    let (trailers_tx, trailers_rx) = oneshot::channel();

    let (want_tx, want_rx) = watch::channel(sender::WANT_READY);

    let tx = Sender {
        want_rx,
        data_tx,
        trailers_tx: Some(trailers_tx),
    };
    let rx = ChannelBody {
        content_length: DecodedLength::CHUNKED,
        want_tx,
        data_rx,
        trailers_rx,
    };

    (tx, rx)
}

impl Body for ChannelBody {
    type Data = Bytes;
    type Error = crate::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.want_tx.send(sender::WANT_READY);

        if !self.data_rx.is_terminated() {
            if let Some(chunk) = ready!(Pin::new(&mut self.data_rx).poll_next(cx)?) {
                self.content_length.sub_if(chunk.len() as u64);
                return Poll::Ready(Some(Ok(Frame::data(chunk))));
            }
        }

        // check trailers after data is terminated
        match ready!(Pin::new(&mut self.trailers_rx).poll(cx)) {
            Ok(t) => Poll::Ready(Some(Ok(Frame::trailers(t)))),
            Err(_) => Poll::Ready(None),
        }
    }

    fn is_end_stream(&self) -> bool {
        self.content_length == DecodedLength::ZERO
    }

    fn size_hint(&self) -> SizeHint {
        let mut hint = SizeHint::default();

        if let Some(content_length) = self.content_length.into_opt() {
            hint.set_exact(content_length);
        }

        hint
    }
}
