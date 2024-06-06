//! Logging for CACvote JX.
//!
//! CACvote JX uses the `tracing` library for logging. After calling [`setup`],
//! you'll be able to call [`tracing::info!`], [`tracing::span!`], and others to
//! print log messages to `stdout` in a flexible and configurable way.
//!
//! You may use the `RUST_LOG` environment variable to configure logging at
//! runtime (see [`EnvFilter`][`tracing_subscriber::EnvFilter`]).

use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use crate::config::Config;

/// Sets up logging for the application. Call this early in the process
/// lifecycle to ensure logs are not silently ignored.
pub(crate) fn setup(config: &Config) -> color_eyre::Result<()> {
    color_eyre::install()?;
    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    format!(
                        "{}={}",
                        env!("CARGO_PKG_NAME").replace('-', "_"),
                        config.log_level
                    )
                    .parse()?,
                )
                .from_env_lossy(),
        )
        .with(stdout_log)
        .init();
    Ok(())
}
