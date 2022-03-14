use anyhow::Result;
use std::io;
pub use tracing::{debug, error, info, trace};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Setup tracing
pub fn install(root: &str, filename: &str) -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
            .with(
                fmt::Layer::new()
                    .with_writer(io::stdout)
                    .with_target(true)
                    // .without_time()
                    .with_file(false),
            )
            .with(
                fmt::Layer::new()
                    .with_writer(tracing_appender::rolling::never(root, filename))
                    .with_target(true)
                    .with_file(false)
                    .with_thread_names(false)
                    .with_thread_ids(false)
                    .with_ansi(false),
            ),
    )
    .map_err(anyhow::Error::from)
}
