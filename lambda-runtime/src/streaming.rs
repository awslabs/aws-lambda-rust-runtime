pub use lambda_runtime_api_client::body::{sender::Sender, Body};

pub use crate::types::StreamResponse as Response;

/// Create a new `Body` stream with associated Sender half.
///
/// Examples
///
/// ```
/// use lambda_runtime::{
///     streaming::{channel, Body, Response},
///     Error, LambdaEvent,
/// };
/// use std::{thread, time::Duration};
///
/// async fn func(_event: LambdaEvent<serde_json::Value>) -> Result<Response<Body>, Error> {
///     let messages = vec!["Hello", "world", "from", "Lambda!"];
///
///     let (mut tx, rx) = channel();
///
///     tokio::spawn(async move {
///         for message in messages.iter() {
///             tx.send_data((message.to_string() + "\n").into()).await.unwrap();
///             thread::sleep(Duration::from_millis(500));
///         }
///     });
///
///     Ok(Response::from(rx))
/// }
/// ```
#[allow(unused)]
#[inline]
pub fn channel() -> (Sender, Body) {
    Body::channel()
}
