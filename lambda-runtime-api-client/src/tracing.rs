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

/// Initialize `tracing-subscriber` with default options.
/// The subscriber uses `RUST_LOG` as the environment variable to determine the log level for your function.
/// It also uses [Lambda's advance logging controls](https://aws.amazon.com/blogs/compute/introducing-advanced-logging-controls-for-aws-lambda-functions/)
/// if they're configured for your function.
/// By default, the log level to emit events is `INFO`.
pub fn init_default_subscriber() {
    let log_format = env::var("AWS_LAMBDA_LOG_FORMAT").unwrap_or_default();
    let log_level = Level::from_str(&env::var("AWS_LAMBDA_LOG_LEVEL").unwrap_or_default()).unwrap_or(Level::INFO);

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
