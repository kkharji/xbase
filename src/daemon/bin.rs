use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use xcodebase::daemon::state::DaemonStateData;
use xcodebase::util::tracing::install_tracing;
use xcodebase::{daemon::*, util::watch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::fs::metadata(DAEMON_SOCKET_PATH).is_ok() {
        std::fs::remove_file(DAEMON_SOCKET_PATH).ok();
    }

    let state: Arc<Mutex<DaemonStateData>> = Default::default();
    let listener = UnixListener::bind(DAEMON_SOCKET_PATH).unwrap();

    install_tracing("/tmp", "xcodebase-daemon.log", tracing::Level::TRACE, true)?;

    tracing::info!("Started");
    loop {
        let state = state.clone();
        if let Ok((mut s, _)) = listener.accept().await {
            tokio::spawn(async move {
                let string = {
                    let mut string = String::default();
                    if let Err(e) = s.read_to_string(&mut string).await {
                        return tracing::error!("[Read Error]: {:?}", e);
                    };
                    string
                };

                if string.is_empty() {
                    return;
                }

                match Request::read(string.clone()) {
                    Err(e) => {
                        return tracing::error!("[Parse Error]: {:?} message: {string}", e);
                    }
                    Ok(req) => {
                        if let Err(e) = req.message.handle(state.clone()).await {
                            return tracing::error!("[Failure]: Cause: ({:?})", e);
                        };
                    }
                };

                update_watchers(state.clone()).await;
            });
        } else {
            anyhow::bail!("Fail to accept a connection")
        };
    }
}

// TODO: Remove wathcers for workspaces that are no longer exist
async fn update_watchers(state: Arc<Mutex<DaemonStateData>>) {
    let copy_state = state.clone();
    let mut current_state = copy_state.lock().await;
    let watched_roots: Vec<String> = current_state.watchers.keys().map(Clone::clone).collect();
    let start_watching: Vec<String> = current_state
        .workspaces
        .keys()
        .filter(|key| !watched_roots.contains(key))
        .map(Clone::clone)
        .collect();

    start_watching.into_iter().for_each(|root| {
        #[cfg(feature = "logging")]
        tracing::info!("Watching {root}");

        let handle = watch::new(root.clone(), state.clone(), watch::recompile_handler);
        current_state.watchers.insert(root, handle);
    });
}
