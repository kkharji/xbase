mod broadcast;
mod error;
mod project;
mod runner;
mod server;
mod state;
mod types;
mod util;
mod watcher;

use {broadcast::*, error::*, project::*, runner::*, state::*, types::*, util::*, watcher::*};

static SOCK_ADDR: &str = "/tmp/xbase.socket";
static PID_PATH: &str = "/tmp/xbase.pid";
static LOG_PATH: &str = "/tmp/xbase.log";

/// Future that await and processes for os signals.
async fn handle_os_signals() -> Result<()> {
    use futures::stream::StreamExt;
    use signal_hook::consts::signal::*;
    use signal_hook_tokio::Signals;

    let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;

    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {}
            SIGINT => {
                tracing::warn!("SERVER STOPPED: Interruption Signal Received");
                break;
            }
            SIGQUIT => {
                tracing::warn!("SERVER STOPPED: Quit Signal Received");
                break;
            }

            SIGTERM => {
                tracing::warn!("SERVER STOPPED: Termination Signal Received");
                break;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[tokio::main]
// TODO: store futures somewhere, to gracefully close connection to clients
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    use fs::cleanup_daemon_runtime;
    use tokio::fs::write;
    use tokio::net::UnixListener;
    use tokio::{pin, select};
    use tracing::info;
    use tracing_setup::setup as tracing_setup;

    let os_signal_handler = tokio::spawn(handle_os_signals());

    let listener = {
        tracing_setup(LOG_PATH, tracing::Level::DEBUG, true)?;
        cleanup_daemon_runtime(PID_PATH, SOCK_ADDR).await?;
        write(PID_PATH, std::process::id().to_string()).await?;
        UnixListener::bind(SOCK_ADDR).unwrap()
    };

    pin!(os_signal_handler);
    info!("SERVER STARTED");

    loop {
        select! {
            Ok((stream, _)) = listener.accept() => tokio::spawn(server::stream::handle(stream)),
            _ = &mut os_signal_handler => break,
        };
    }

    drop(listener);

    cleanup_daemon_runtime(PID_PATH, SOCK_ADDR).await?;

    Ok(())
}
