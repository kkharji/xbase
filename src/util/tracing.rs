use anyhow::Result;
use std::io;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Setup tracing
pub fn install_tracing(root: &str, filename: &str) -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(Level::TRACE.into()))
            .with(
                fmt::Layer::new()
                    .with_writer(io::stdout)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(false),
            )
            .with(
                fmt::Layer::new()
                    .with_writer(rolling::never(root, filename))
                    .with_target(true)
                    .with_file(false)
                    .with_thread_names(false)
                    .with_thread_ids(false)
                    .with_ansi(false),
            ),
    )
    .map_err(anyhow::Error::from)
}
