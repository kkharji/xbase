#![allow(dead_code)]
use crate::{client::Client, constants::DAEMON_STATE, Error, Result};
use process_stream::{Process, StreamExt};
use tokio::task::JoinHandle;

/// Run Service Task Handler
pub enum RunServiceHandler {
    // Runner is running successfully
    Running((Process, JoinHandle<Result<()>>)),
    // Runner Errored
    Errored(Error),
    // Runner Stopped
    Stopped(i32),
}

impl RunServiceHandler {
    // Change the status of the process to running
    pub fn new(client: Client, mut process: Process) -> Result<Self> {
        let mut stream = process.spawn_and_stream()?;
        let kill_send = process.clone_kill_sender().unwrap();

        let handler = tokio::spawn(async move {
            while let Some(output) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let ref state = state.lock().await;
                let ref mut logger = match client.nvim(state) {
                    Ok(nvim) => nvim.logger(),
                    Err(_) => {
                        // TODO: Update state to set current handler as Errored
                        tracing::info!("Nvim Instance closed, closing runner ..");
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

            // TODO: Update state to set current handler as stopped
            Ok(())
        });
        Ok(Self::Running((process, handler)))
    }
}
