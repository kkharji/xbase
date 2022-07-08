mod broadcast;
mod error;
mod project;
mod runner;
mod server;
mod state;
mod types;
mod util;
mod watcher;

use futures::{FutureExt, SinkExt, TryStreamExt};
use once_cell::sync::Lazy;
use tap::Pipe;
use tokio::net::UnixListener;
use tracing::Level;
use util::pid;
use xcodeproj::pbxproj::PBXTargetPlatform;
use {
    broadcast::*, error::*, project::*, runner::*, server::*, state::*, types::*, util::*,
    watcher::*,
};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sock_addr = "/tmp/xbase.socket";
    let pid_path = "/tmp/xbase.pid";

    tracing_setup::setup("/tmp", "xbase-daemon.log", Level::DEBUG, true)?;
    fs::ensure_one_instance(pid_path, sock_addr).await?;

    let listener = UnixListener::bind(sock_addr).unwrap();

    tracing::info!("Server Started!");

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            tracing::debug!("Received new connection");
            tokio::spawn(async move {
                let (mut reader, mut writer) = server::stream::split(&mut stream);
                loop {
                    match reader.try_next().await {
                        Ok(Some(request)) => {
                            request
                                .handle()
                                .then(|res| writer.send(res))
                                .await
                                .map_err(|err| tracing::error!("Fail to send response: {err}"))
                                .ok();
                        }
                        Ok(None) => {
                            tracing::info!("Closing a connection to a client");
                            break;
                        }
                        Err(err) => tracing::error!("Fail to read request: {err}"),
                    };
                }
            });
        } else {
            tracing::error!("Fail to accept a connection")
        };
    }
}
