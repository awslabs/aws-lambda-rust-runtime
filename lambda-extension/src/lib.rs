#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions, clippy::type_complexity)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! This module includes utilities to create Lambda Runtime Extensions.
//!
//! Create a type that conforms to the [`Extension`] trait. This type can then be passed
//! to the the `lambda_extension::run` function, which launches and runs the Lambda runtime extension.
use std::{fmt, future::Future};
pub use tower::{self, make::Shared as SharedService, service_fn, Service};

mod error;
pub use error::*;
mod extension;
pub use extension::*;
mod events;
pub use events::*;
mod logs;
pub use logs::*;
mod telemetry;
pub use telemetry::*;

/// Include several request builders to interact with the Extension API.
pub mod requests;

/// Execute the given events processor
pub async fn run<E>(events_processor: E) -> Result<(), Error>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display + fmt::Debug,
{
    let ext = Extension::new().with_events_processor(events_processor);
    ext.run().await
}
