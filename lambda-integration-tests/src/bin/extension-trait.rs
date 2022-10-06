use std::{
    future::{ready, Future},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};

use lambda_extension::{Error, LambdaEvent, NextEvent, Service};
use tracing::info;

struct MyExtension {
    invoke_count: usize,
    ready: AtomicBool,
}

impl Default for MyExtension {
    fn default() -> Self {
        Self {
            invoke_count: usize::default(),
            // New instances are not ready to be called until polled.
            ready: false.into(),
        }
    }
}

impl Clone for MyExtension {
    fn clone(&self) -> Self {
        Self {
            invoke_count: self.invoke_count,
            // Cloned instances may not be immediately ready to be called.
            // https://docs.rs/tower/0.4.13/tower/trait.Service.html#be-careful-when-cloning-inner-services
            ready: false.into(),
        }
    }
}

impl Service<LambdaEvent> for MyExtension {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    type Response = ();

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        if self.ready.swap(true, Ordering::SeqCst) {
            info!("[extension] Service was already ready");
        } else {
            info!("[extension] Service is now ready");
        };

        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: LambdaEvent) -> Self::Future {
        match event.next {
            NextEvent::Shutdown(e) => {
                info!("[extension] Shutdown event received: {:?}", e);
            }
            NextEvent::Invoke(e) => {
                self.invoke_count += 1;
                info!("[extension] Request event {} received: {:?}", self.invoke_count, e);
            }
        }

        // After being called once, the service is no longer ready until polled again.
        if self.ready.swap(false, Ordering::SeqCst) {
            info!("[extension] The service is ready");
        } else {
            // https://docs.rs/tower/latest/tower/trait.Service.html#backpressure
            // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
            // > Services are permitted to panic if `call` is invoked without obtaining
            // > `Poll::Ready(Ok(()))` from `poll_ready`.
            panic!("[extension] The service is not ready; `.poll_ready()` must be called first");
        }

        Box::pin(ready(Ok(())))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_extension::run(MyExtension::default()).await
}
