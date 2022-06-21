mod bin;
mod handler;
mod service;
mod simulator;

use crate::constants::DAEMON_STATE;
use crate::nvim::Logger;
use crate::{RequestHandler, Result};
use async_trait::async_trait;
use process_stream::Process;
use xbase_proto::RunRequest;

pub use service::RunService;
pub use {bin::*, simulator::*};

#[async_trait::async_trait]
pub trait Runner {
    /// Run Project
    async fn run<'a>(&self, logger: &mut Logger<'a>) -> Result<Process>;
}

#[async_trait]
impl RequestHandler for RunRequest {
    async fn handle(self) -> Result<()>
    where
        Self: Sized + std::fmt::Debug,
    {
        log::info!("⚙️ Running: {}", self.settings.to_string());

        let ref key = self.to_string();
        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if self.ops.is_once() {
            // TODO(run): might want to keep track of ran services
            RunService::new(state, self).await?;
            return Ok(());
        }

        let client = self.client.clone();
        if self.ops.is_watch() {
            let watcher = state.watcher.get(&self.client.root)?;
            if watcher.contains_key(key) {
                state
                    .clients
                    .get(&self.client.pid)?
                    .echo_err("Already watching with {key}!!")
                    .await?;
            } else {
                let pid = self.client.pid.to_owned();
                let run_service = RunService::new(state, self).await?;
                let watcher = state.watcher.get_mut(&client.root)?;
                watcher.add(run_service)?;
                state.clients.get(&pid)?.set_watching(true).await?;
            }
        } else {
            let watcher = state.watcher.get_mut(&self.client.root)?;
            let listener = watcher.remove(&self.to_string())?;
            state
                .clients
                .get(&self.client.pid)?
                .set_watching(false)
                .await?;
            listener.discard(state).await?;
        }

        state.sync_client_state().await?;

        Ok(())
    }
}
