//! This module provides primitives to work with `tracing`
//! and `tracing-subscriber` in Lambda functions.
//!
//! The `tracing` and `tracing-subscriber` crates are re-exported
//! so you don't have to include them as direct dependencies in
//! your projects.

use std::{env, str::FromStr};

use subscriber::filter::{EnvFilter, LevelFilter};
/// Re-export the `tracing` crate to have access to tracing macros
/// like `info!`, `debug!`, `trace!` and so on.
pub use tracing::*;

/// Re-export the `tracing-subscriber` crate to build your own subscribers.
pub use tracing_subscriber as subscriber;

const DEFAULT_LOG_LEVEL: &str = "INFO";

/// Initialize `tracing-subscriber` with default logging options.
///
/// This function uses environment variables set with [Lambda's advance logging controls](https://aws.amazon.com/blogs/compute/introducing-advanced-logging-controls-for-aws-lambda-functions/)
/// if they're configured for your function.
///
/// This subscriber sets the logging level based on environment variables:
///     - if `AWS_LAMBDA_LOG_LEVEL` is set, it takes precedence over any other environment variables.
///     - if `AWS_LAMBDA_LOG_LEVEL` is not set, check if `RUST_LOG` is set.
///     - if none of those two variables are set, use `INFO` as the logging level.
///
/// The logging format can also be changed based on Lambda's advanced logging controls.
/// If the `AWS_LAMBDA_LOG_FORMAT` environment variable is set to `JSON`, the log lines will be formatted as json objects,
/// otherwise they will be formatted with the default tracing format.
pub fn init_default_subscriber() {
    let log_format = env::var("AWS_LAMBDA_LOG_FORMAT").unwrap_or_default();
    let log_level_str = env::var("AWS_LAMBDA_LOG_LEVEL").or_else(|_| env::var("RUST_LOG"));
    let log_level = Level::from_str(log_level_str.as_deref().unwrap_or(DEFAULT_LOG_LEVEL)).unwrap_or(Level::INFO);

    let collector = tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::from_level(log_level).into())
                .from_env_lossy(),
        );

    if log_format.eq_ignore_ascii_case("json") {
        collector.json().init()
    } else {
        collector.init()
    }
}
