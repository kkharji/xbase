use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use xbase::constants::*;
use xbase::daemon::*;
use xbase::util::tracing::install_tracing;

use tracing::*;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::fs::metadata(DAEMON_SOCKET_PATH).is_ok() {
        std::fs::remove_file(DAEMON_SOCKET_PATH).ok();
    }

    let listener = UnixListener::bind(DAEMON_SOCKET_PATH).unwrap();

    install_tracing("/tmp", "xbase-daemon.log", Level::TRACE, true)?;

    tracing::info!("Started");
    loop {
        if let Ok((mut s, _)) = listener.accept().await {
            tokio::spawn(async move {
                let msg = {
                    let mut msg = String::default();
                    if let Err(e) = s.read_to_string(&mut msg).await {
                        return error!("[Read Error]: {:?}", e);
                    };
                    msg
                };

                if msg.is_empty() {
                    return;
                }

                let req = match Request::read(msg.clone()) {
                    Err(e) => {
                        return error!("[Parse Error]: {:?} message: {msg}", e);
                    }
                    Ok(req) => req,
                };

                if let Err(e) = req.message.handle().await {
                    return error!("[Failure]: Cause: ({:?})", e);
                };

                let state = DAEMON_STATE.clone();
                let mut state = state.lock().await;
                state.validate().await;

                // update_watchers(state.clone()).await;
            });
        } else {
            anyhow::bail!("Fail to accept a connection")
        };
    }
}
