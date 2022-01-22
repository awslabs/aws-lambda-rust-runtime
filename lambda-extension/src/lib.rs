#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//! This module includes utilities to create Lambda Runtime Extensions.
//!
//! Create a type that conforms to the [`Extension`] trait. This type can then be passed
//! to the the `lambda_extension::run` function, which launches and runs the Lambda runtime extension.
use std::{fmt, future::Future};
pub use tower::{self, service_fn, Service};

mod error;
pub use error::*;
mod events;
pub use events::*;

/// Include several request builders to interact with the Extension API.
pub mod requests;

mod runtime;
pub use runtime::*;

/// Execute the given extension
pub async fn run<E>(extension: E) -> Result<(), Error>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display,
{
    Runtime::builder().register().await?.run(extension).await
}
