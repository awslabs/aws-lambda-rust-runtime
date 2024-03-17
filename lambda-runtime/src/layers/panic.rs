use crate::{Diagnostic, LambdaEvent};
use futures::{future::CatchUnwind, FutureExt};
use pin_project::pin_project;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task;
use tower::Service;
use tracing::error;

/// Tower service that transforms panics into an error. Panics are converted to errors both when
/// constructed in [tower::Service::call] and when constructed in the returned
/// [tower::Service::Future].
///
/// This type is only meant for internal use in the Lambda runtime crate. It neither augments the
/// inner service's request type, nor its response type. It merely transforms the error type
/// from `Into<Diagnostic<'_> + Debug` into `Diagnostic<'a>` to turn panics into diagnostics.
#[derive(Clone)]
pub struct CatchPanicService<'a, S> {
    inner: S,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, S> CatchPanicService<'a, S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, S, Payload> Service<LambdaEvent<Payload>> for CatchPanicService<'a, S>
where
    S: Service<LambdaEvent<Payload>>,
    S::Future: 'a,
    S::Error: Into<Diagnostic<'a>> + Debug,
{
    type Error = Diagnostic<'a>;
    type Response = S::Response;
    type Future = CatchPanicFuture<'a, S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: LambdaEvent<Payload>) -> Self::Future {
        // Catch panics that result from calling `call` on the service
        let task = std::panic::catch_unwind(AssertUnwindSafe(|| self.inner.call(req)));

        // Catch panics that result from polling the future returned from `call`
        match task {
            Ok(task) => {
                let fut = AssertUnwindSafe(task).catch_unwind();
                CatchPanicFuture::Future(fut, PhantomData)
            }
            Err(err) => {
                error!(error = ?err, "user handler panicked");
                CatchPanicFuture::Error(err)
            }
        }
    }
}

/// Future returned by [CatchPanicService].
#[pin_project(project = CatchPanicFutureProj)]
pub enum CatchPanicFuture<'a, F> {
    Future(#[pin] CatchUnwind<AssertUnwindSafe<F>>, PhantomData<&'a ()>),
    Error(Box<dyn Any + Send + 'static>),
}

impl<'a, F, T, E> Future for CatchPanicFuture<'a, F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Diagnostic<'a>> + Debug,
{
    type Output = Result<T, Diagnostic<'a>>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        use task::Poll;
        match self.project() {
            CatchPanicFutureProj::Future(fut, _) => match fut.poll(cx) {
                Poll::Ready(ready) => match ready {
                    Ok(inner_result) => Poll::Ready(inner_result.map_err(|err| err.into())),
                    Err(err) => {
                        error!(error = ?err, "user handler panicked");
                        Poll::Ready(Err(Self::build_panic_diagnostic(&err)))
                    }
                },
                Poll::Pending => Poll::Pending,
            },
            CatchPanicFutureProj::Error(err) => Poll::Ready(Err(Self::build_panic_diagnostic(err))),
        }
    }
}

impl<'a, F> CatchPanicFuture<'a, F> {
    fn build_panic_diagnostic(err: &Box<dyn Any + Send>) -> Diagnostic<'a> {
        let error_type = std::any::type_name_of_val(&err);
        let msg = if let Some(msg) = err.downcast_ref::<&str>() {
            format!("Lambda panicked: {msg}")
        } else {
            "Lambda panicked".to_string()
        };
        Diagnostic {
            error_type: Cow::Borrowed(error_type),
            error_message: Cow::Owned(msg),
        }
    }
}
