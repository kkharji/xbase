#![allow(dead_code)]
use crate::{constants::DAEMON_STATE, Result};
use process_stream::ProcessExt;
use process_stream::{Process, StreamExt};
use tokio::task::JoinHandle;
use xbase_proto::Client;

/// Run Service Task Handler
pub struct RunServiceHandler {
    process: Process,
    inner: JoinHandle<Result<()>>,
}

impl RunServiceHandler {
    // Change the status of the process to running
    pub fn new(
        key: &String,
        target: &String,
        client: &Client,
        mut process: Process,
    ) -> Result<Self> {
        let (key, target, client) = (key.clone(), target.clone(), client.clone());
        let mut stream = process.spawn_and_stream()?;
        let abort = process.aborter().unwrap();

        let inner = tokio::spawn(async move {
            // TODO: find a better way to close this!
            //
            // Right now it just wait till the user try print something
            while let Some(output) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let ref mut state = state.lock().await;
                let ref mut logger = match state.clients.get(&client.pid) {
                    Ok(nvim) => nvim.logger(),
                    Err(_) => {
                        log::warn!("Nvim Instance closed, closing runner ..");
                        state.watcher.get_mut(&client.root)?.listeners.remove(&key);
                        abort.notify_waiters();
                        break;
                    }
                };

                logger.set_title(format!("Run:{target}"));

                use process_stream::ProcessItem::*;
                match output {
                    Output(msg) => {
                        if !msg.contains("ignoring singular matrix") {
                            logger.append(msg).await?;
                        }
                    }
                    Error(msg) => {
                        logger.append(format!("[Error] {msg}")).await?;
                    }
                    // TODO: this should be skipped when user re-run the app
                    Exit(code) => {
                        let success = &code == "0";
                        if success {
                            logger.append(format!("disconnected")).await?;
                        } else {
                            logger
                                .append(format!("[Error]: disconnected, exit: {code}"))
                                .await?;
                        }
                        logger.set_status_end(success, !success).await?;

                        log::info!("[target: {target}] runner closed");
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
