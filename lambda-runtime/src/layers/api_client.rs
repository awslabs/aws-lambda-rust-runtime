use crate::LambdaInvocation;
use futures::{future::BoxFuture, FutureExt};
use lambda_runtime_api_client::{body::Body, BoxError, Client};
use std::future::Future;
use std::sync::Arc;
use std::task;
use tower::Service;
use tracing::error;

/// Tower service that sends a Lambda Runtime API response to the Lambda Runtime HTTP API using
/// a previously initialized client.
///
/// This type is only meant for internal use in the Lambda runtime crate. It neither augments the
/// inner service's request type nor its error type. However, this service returns an empty
/// response `()` as the Lambda request has been completed.
pub struct RuntimeApiClientService<S> {
    inner: S,
    client: Arc<Client>,
}

impl<S> RuntimeApiClientService<S> {
    pub fn new(inner: S, client: Arc<Client>) -> Self {
        Self { inner, client }
    }
}

impl<S> Service<LambdaInvocation> for RuntimeApiClientService<S>
where
    S: Service<LambdaInvocation, Error = BoxError>,
    S::Future: Future<Output = Result<http::Request<Body>, BoxError>> + Send + 'static,
{
    type Response = ();
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<(), BoxError>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let request_fut = self.inner.call(req);
        let client = self.client.clone();
        async move {
            let request = request_fut.await?;
            client.call(request).await.map_err(|err| {
                error!(error = ?err, "failed to send request to Lambda Runtime API");
                err
            })?;
            Ok(())
        }
        .boxed()
    }
}
