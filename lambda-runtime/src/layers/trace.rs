use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument};

use crate::{Context, LambdaInvocation};
use lambda_runtime_api_client::BoxError;
use std::task;

/// Tower middleware to create a tracing span for invocations of the Lambda function.
#[derive(Default)]
pub struct TracingLayer {}

impl TracingLayer {
    /// Create a new tracing layer.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> Layer<S> for TracingLayer {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService { inner }
    }
}

/// Tower service returned by [TracingLayer].
pub struct TracingService<S> {
    inner: S,
}

impl<S> Service<LambdaInvocation> for TracingService<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
{
    type Response = ();
    type Error = BoxError;
    type Future = Instrumented<S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = request_span(&req.context);
        let future = {
            let _guard = span.enter();
            self.inner.call(req)
        };
        future.instrument(span)
    }
}

/* ------------------------------------------- UTILS ------------------------------------------- */

fn request_span(ctx: &Context) -> tracing::Span {
    match &ctx.xray_trace_id {
        Some(trace_id) => {
            tracing::info_span!(
                "Lambda runtime invoke",
                requestId = &ctx.request_id,
                xrayTraceId = trace_id
            )
        }
        None => {
            tracing::info_span!("Lambda runtime invoke", requestId = &ctx.request_id)
        }
    }
}
