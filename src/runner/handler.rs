use crate::*;
use process_stream::ProcessExt;
use process_stream::{Process, StreamExt};
use std::sync::Weak;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

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
        mut process: Process,
        broadcast: Weak<Broadcast>,
        watcher: Weak<Mutex<WatchService>>,
    ) -> Result<Self> {
        let (key, target) = (key.clone(), target.clone());
        let mut stream = process.spawn_and_stream()?;
        let abort = process.aborter().unwrap();

        let inner: _ = tokio::spawn(async move {
            // TODO: find a better way to close this!
            //
            // Right now it just wait till the user try print something
            while let Some(output) = stream.next().await {
                let ref mut broadcast = match broadcast.upgrade() {
                    Some(broadcast) => broadcast,
                    None => {
                        tracing::warn!("No client instance listening, closing runner ..");
                        if let Some(watcher) = watcher.upgrade() {
                            watcher.lock().await.listeners.remove(&key);
                        }
                        abort.notify_waiters();
                        break;
                    }
                };

                use process_stream::ProcessItem::*;
                match output {
                    Output(msg) => {
                        if !msg.contains("ignoring singular matrix") {
                            broadcast.log_info(msg);
                        }
                    }
                    Error(msg) => {
                        broadcast.log_error(msg);
                    }
                    // TODO: this should be skipped when user re-run the app
                    Exit(code) => {
                        let success = &code == "0";
                        if success {
                            broadcast.log_info("disconnected");
                            broadcast.update_statusline(StatuslineState::Success);
                        } else {
                            broadcast.log_error(format!("disconnected, exit: {code}"));
                            broadcast.update_statusline(StatuslineState::Failure);
                        }

                        tracing::info!("[{target}] Runner Closed");
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
