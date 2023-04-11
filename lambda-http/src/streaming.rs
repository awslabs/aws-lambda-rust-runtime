use crate::tower::ServiceBuilder;
use crate::Request;
use crate::{request::LambdaRequest, RequestExt};
pub use aws_lambda_events::encodings::Body as LambdaEventBody;
use bytes::Bytes;
pub use http::{self, Response};
use http_body::Body;
use lambda_runtime::LambdaEvent;
pub use lambda_runtime::{self, service_fn, tower, Context, Error, Service};
use std::fmt::{Debug, Display};

/// Starts the Lambda Rust runtime and stream response back [Configure Lambda
/// Streaming Response](https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html).
///
/// This takes care of transforming the LambdaEvent into a [`Request`] and
/// accepts [`http::Response<http_body::Body>`] as response.
pub async fn run_with_streaming_response<'a, S, B, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = Response<B>, Error = E>,
    S::Future: Send + 'a,
    E: Debug + Display,
    B: Body + Unpin + Send + 'static,
    B::Data: Into<Bytes> + Send,
    B::Error: Into<Error> + Send + Debug,
{
    let svc = ServiceBuilder::new()
        .map_request(|req: LambdaEvent<LambdaRequest>| {
            let event: Request = req.payload.into();
            event.with_lambda_context(req.context)
        })
        .service(handler);

    lambda_runtime::run_with_streaming_response(svc).await
}
