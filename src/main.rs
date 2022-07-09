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

/// Future that await and processes for os signals.
async fn handle_os_signals() -> Result<()> {
    use futures::stream::StreamExt;
    use signal_hook::consts::signal::*;
    use signal_hook_tokio::Signals;

    let sep = util::fmt::separator();
    let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;

    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {}
            SIGINT => {
                tracing::warn!("\n{sep}\nServer Stopped: Interruption Signal (Ctrl + C)\n{sep}");
                break;
            }
            SIGQUIT => {
                tracing::warn!("{sep}\nServer Stopped: Quit Signal (Ctrl + D)\n{sep}");
                break;
            }

            SIGTERM => {
                tracing::warn!("\n{sep}\nServer Stopped: Termination Signal\n{sep}");
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
    let sock_addr = "/tmp/xbase.socket";
    let pid_path = "/tmp/xbase.pid";
    let sep = util::fmt::separator();
    let listener = {
        tracing_setup("/tmp", "xbase-daemon.log", tracing::Level::DEBUG, true)?;
        cleanup_daemon_runtime(pid_path, sock_addr).await?;
        write(pid_path, std::process::id().to_string()).await?;
        UnixListener::bind(sock_addr).unwrap()
    };

    pin!(os_signal_handler);
    info!("\n{sep}\nServer Started\n{sep}");

    loop {
        select! {
            Ok((stream, _)) = listener.accept() => tokio::spawn(server::handle(stream)),
            _ = &mut os_signal_handler => break,
        };
    }

    drop(listener);

    cleanup_daemon_runtime(pid_path, sock_addr).await?;

    Ok(())
}
