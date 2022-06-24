use std::io;
pub use tracing::dispatcher::SetGlobalDefaultError;
use tracing::subscriber::set_global_default;
use tracing_appender::rolling;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{registry, EnvFilter};

pub use tracing::*;
pub use tracing_attributes;
pub use tracing_subscriber::*;

/// Setup tracing
pub fn setup(
    root: &str,
    filename: &str,
    default_level: Level,
    with_stdout: bool,
) -> Result<(), SetGlobalDefaultError> {
    let default_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::from_default_env().add_directive(default_level.into()))
        .add_directive("tarpc::server=warn".parse().unwrap());

    let fmt_file = Layer::new()
        .with_writer(rolling::never(root, filename))
        .with_target(true)
        .with_file(false)
        .without_time()
        .with_thread_names(false)
        .with_thread_ids(false)
        // .with_ansi(false)
        .compact();
    let fmt_stdout = Layer::new()
        .with_writer(io::stdout)
        .with_target(true)
        .with_line_number(true)
        .without_time()
        .with_file(false)
        .compact();

    if with_stdout {
        set_global_default(
            registry()
                .with(default_filter)
                .with(fmt_file)
                .with(fmt_stdout),
        )?
    } else {
        set_global_default(registry().with(default_filter).with(fmt_file))?
    }
    Ok(())
}
