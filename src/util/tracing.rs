use anyhow::Result;
use std::io;
use tracing::subscriber::set_global_default;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, registry, EnvFilter};

/// Setup tracing
pub fn install_tracing(
    root: &str,
    filename: &str,
    default_level: Level,
    with_stdout: bool,
) -> Result<()> {
    let default_filter = EnvFilter::from_default_env().add_directive(default_level.into());
    let fmt_file = fmt::Layer::new()
        .with_writer(rolling::never(root, filename))
        .with_target(true)
        .with_file(false)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_ansi(false);
    let fmt_stdout = fmt::Layer::new()
        .with_writer(io::stdout)
        .with_target(true)
        .with_line_number(true)
        .with_file(false);

    if with_stdout {
        set_global_default(
            registry()
                .with(default_filter)
                .with(fmt_file)
                .with(fmt_stdout),
        )
        .map_err(anyhow::Error::from)
    } else {
        set_global_default(registry().with(default_filter).with(fmt_file))
            .map_err(anyhow::Error::from)
    }
}
