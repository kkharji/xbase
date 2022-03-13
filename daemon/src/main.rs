use xcodebase::constants::DAEMON_SOCKET_PATH;
use xcodebase::state::{SharedState, State};
use xcodebase::tracing::{self, error, info};
use xcodebase::{watch, Command};

use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;

use anyhow::Result;
use std::fs;

fn create_listener() -> std::io::Result<UnixListener> {
    if fs::metadata(DAEMON_SOCKET_PATH).is_ok() {
        fs::remove_file(DAEMON_SOCKET_PATH)?
    }
    info!("Started");
    UnixListener::bind(DAEMON_SOCKET_PATH)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::install("/tmp", "xcodebase-daemon.log")?;
    let state: SharedState = State::new()?;
    let listener = create_listener()?;
    loop {
        let state = state.clone();
        let (mut s, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // let mut current_state = state.lock().await;
            // current_state.update_clients();

            // trace!("Current State: {:?}", state.lock().await)
            let mut string = String::default();

            if let Err(e) = s.read_to_string(&mut string).await {
                error!("[Read Error]: {:?}", e);
                return;
            };

            if string.len() == 0 {
                return;
            }

            let msg = Command::parse(string.as_str().trim());

            if let Err(e) = msg {
                error!("[Parse Error]: {:?}", e);
                return;
            };

            let msg = msg.unwrap();
            if let Err(e) = msg.handle(state.clone()).await {
                error!("[Handling Error]: {:?}", e);
                return;
            };

            watch::update(state, msg).await;
        });
    }
}
