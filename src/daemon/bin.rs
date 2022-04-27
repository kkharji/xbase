use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use xcodebase::util::tracing::install_tracing;
use xcodebase::{daemon::*, util::watch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::fs::metadata(DAEMON_SOCKET_PATH).is_ok() {
        std::fs::remove_file(DAEMON_SOCKET_PATH).ok();
    }

    let state: Arc<Mutex<state::DaemonStateData>> = Default::default();
    let listener = tokio::net::UnixListener::bind(DAEMON_SOCKET_PATH).unwrap();

    install_tracing("/tmp", "xcodebase-daemon.log", tracing::Level::TRACE, true)?;

    tracing::info!("Started");
    loop {
        let state = state.clone();
        if let Ok((mut s, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut string = String::default();

                if let Err(e) = s.read_to_string(&mut string).await {
                    tracing::error!("[Read Error]: {:?}", e);
                    return;
                };

                if string.len() == 0 {
                    return;
                }

                let msg = match DaemonRequest::parse(string.as_str().trim()) {
                    Err(e) => {
                        tracing::error!("[Parse Error]: {:?}", e);
                        return;
                    }
                    Ok(msg) => msg,
                };

                if let Err(e) = msg.handle(state.clone()).await {
                    tracing::error!("[Failure]: Cause: ({:?}), Message: {:?}", e, msg);
                    return;
                };

                // watch::update(state, msg).await;

                let copy = state.clone();
                let mut current_state = copy.lock().await;
                // let mut watched_roots: Vec<String> = vec![];
                let mut start_watching: Vec<String> = vec![];

                // TODO: Remove wathcers for workspaces that are no longer exist

                let watched_roots = current_state
                    .watchers
                    .keys()
                    .map(Clone::clone)
                    .collect::<Vec<String>>();

                for key in current_state.workspaces.keys() {
                    if !watched_roots.contains(key) {
                        start_watching.push(key.clone());
                    }
                }

                for root in start_watching {
                    let handle = watch::handler(state.clone(), root.clone());
                    #[cfg(feature = "logging")]
                    tracing::info!("Watching {root}");
                    current_state.watchers.insert(root, handle);
                }
            });
        } else {
            anyhow::bail!("Fail to accept a connection")
        };
    }
}
