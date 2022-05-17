use anyhow::Result;
use tap::Pipe;
use tokio::fs::{metadata, read_to_string, remove_file, write};
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tracing::*;
use xbase::util::{proc_kill, tracing::install_tracing};
use xbase::{constants::*, daemon::*};

#[tokio::main]
async fn main() -> Result<()> {
    ensure_single_instance().await?;

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

async fn ensure_single_instance() -> Result<()> {
    if metadata(DAEMON_SOCKET_PATH).await.ok().is_some() {
        remove_file(DAEMON_SOCKET_PATH).await.ok();
        if metadata(DAEMON_PID_PATH).await.ok().is_some() {
            read_to_string(DAEMON_PID_PATH)
                .await?
                .pipe_ref(proc_kill)
                .await?;
        }
        remove_file(DAEMON_PID_PATH).await.ok();
    }
    write(DAEMON_PID_PATH, std::process::id().to_string()).await?;
    Ok(())
}
