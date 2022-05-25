#![allow(dead_code)]
use crate::{client::Client, constants::DAEMON_STATE, Result};
use process_stream::{Process, StreamExt};
use tokio::task::JoinHandle;

/// Run Service Task Handler
pub struct RunServiceHandler {
    process: Process,
    inner: JoinHandle<Result<()>>,
}

impl RunServiceHandler {
    // Change the status of the process to running
    pub fn new(client: Client, mut process: Process, key: String) -> Result<Self> {
        let mut stream = process.spawn_and_stream()?;
        let kill_send = process.clone_kill_sender().unwrap();

        let inner = tokio::spawn(async move {
            // TODO:
            while let Some(output) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let ref mut state = state.lock().await;
                let ref mut logger = match client.nvim(state) {
                    Ok(nvim) => nvim.logger(),
                    Err(_) => {
                        tracing::info!("Nvim Instance closed, closing runner ..");
                        state.watcher.get_mut(&client.root)?.listeners.remove(&key);
                        kill_send.send(()).await.ok();
                        break;
                    }
                };

                use process_stream::ProcessItem::*;
                match output {
                    Output(msg) => {
                        if !msg.contains("ignoring singular matrix") {
                            logger.log(msg).await?;
                        }
                    }
                    Error(msg) => {
                        logger.log(format!("[Error] {msg}")).await?;
                    }
                    // TODO: this should be skipped when user re-run the app
                    Exit(code) => {
                        let success = &code == "0";
                        logger.log(format!("[Exit] {code}")).await?;
                        logger.set_status_end(success, !success).await?;
                        break;
                    }
                };
            }

            drop(stream);

            Ok(())
        });

        Ok(Self { process, inner })
    }

    /// Get a reference to the run service handler's process.
    #[must_use]
    pub fn process(&self) -> &Process {
        &self.process
    }

    /// Get a reference to the run service handler's handler.
    #[must_use]
    pub fn inner(&self) -> &JoinHandle<Result<()>> {
        &self.inner
    }
}
