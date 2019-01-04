#![warn(missing_docs)]
#![deny(warnings)]
//! The Lambda runtime core crate implements [Lambda's custom runtime main loop](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html#runtimes-custom-build).
//! The crate receives a `Handler` type that consumed events in the form of `Vec<u8>` and
//! outputs a `Result` with a `Vec<u8>` successful output.
//!
//! **Unless you have specific requirements to consume/produce raw bytes, you should look at the
//! [`lambda_runtime` crate](https://crates.io/crates/lambda_runtime)**.
//!
//! TODO: Add example

mod context;
mod env;
mod error;
mod handler;
mod runtime;

pub use crate::{
    context::Context,
    env::{ConfigProvider, EnvConfigProvider},
    handler::Handler,
    runtime::*,
};

pub use lambda_runtime_errors::{HandlerError, LambdaErrorExt, LambdaResultExt};
