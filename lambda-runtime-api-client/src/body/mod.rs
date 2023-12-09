//! HTTP body utilities. Extracted from Axum under MIT license.
//! https://github.com/tokio-rs/axum/blob/main/axum/LICENSE

use crate::{BoxError, Error};
use bytes::Bytes;
use futures_util::stream::Stream;
use http_body::{Body as _, Frame};
use http_body_util::{BodyExt, Collected};
use std::pin::Pin;
use std::task::{Context, Poll};

use self::channel::Sender;

macro_rules! ready {
    ($e:expr) => {
        match $e {
            std::task::Poll::Ready(v) => v,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

mod channel;
mod sender;
mod watch;

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, Error>;

fn boxed<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    try_downcast(body).unwrap_or_else(|body| body.map_err(Error::new).boxed_unsync())
}

pub(crate) fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

/// The body type used in axum requests and responses.
#[derive(Debug)]
pub struct Body(BoxBody);

impl Body {
    /// Create a new `Body` that wraps another [`http_body::Body`].
    pub fn new<B>(body: B) -> Self
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
    {
        try_downcast(body).unwrap_or_else(|body| Self(boxed(body)))
    }

    /// Create an empty body.
    pub fn empty() -> Self {
        Self::new(http_body_util::Empty::new())
    }

    /// Create a new `Body` stream with associated Sender half.
    pub fn channel() -> (Sender, Body) {
        let (sender, body) = channel::channel();
        (sender, Body::new(body))
    }

    // /// Create a new `Body` from a [`Stream`].
    // ///
    // /// [`Stream`]: https://docs.rs/futures-core/latest/futures_core/stream/trait.Stream.html
    // pub fn from_stream<S>(stream: S) -> Self
    // where
    //     S: TryStream + Send + 'static,
    //     S::Ok: Into<Bytes>,
    //     S::Error: Into<BoxError>,
    // {
    //     Self::new(StreamBody {
    //         stream: SyncWrapper::new(stream),
    //     })
    // }

    /// Collect the body into `Bytes`
    pub async fn collect(self) -> Result<Collected<Bytes>, Error> {
        self.0.collect().await
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(http_body_util::Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(std::borrow::Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(std::borrow::Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.0).poll_frame(cx)
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        self.0.size_hint()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }
}

impl Stream for Body {
    type Item = Result<Bytes, Error>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match futures_util::ready!(Pin::new(&mut self).poll_frame(cx)?) {
                Some(frame) => match frame.into_data() {
                    Ok(data) => return Poll::Ready(Some(Ok(data))),
                    Err(_frame) => {}
                },
                None => return Poll::Ready(None),
            }
        }
    }
}
