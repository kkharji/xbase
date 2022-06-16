use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

use {
    crate::constants::DAEMON_STATE,
    crate::state::State,
    crate::util::serde::value_or_default,
    crate::watch::{Event, Watchable},
    tokio::sync::MutexGuard,
    xclog::XCLogger,
};

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub client: Client,
    pub settings: BuildConfiguration,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    #[serde(deserialize_with = "value_or_default")]
    pub ops: RequestOps,
}

#[async_trait]
impl Handler for BuildRequest {
    async fn handle(self) -> Result<()> {
        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        match self.ops {
            RequestOps::Once => self.trigger(state, &Event::default()).await?,
            _ => {
                let watcher = self.client.get_watcher_mut(state)?;
                if self.ops.is_watch() {
                    watcher.add(self)?;
                } else {
                    watcher.remove(&self.to_string())?;
                }
                state.sync_client_state().await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        tracing::info!("Building {}", self.client.abbrev_root());
        let is_once = self.ops.is_once();
        let (root, config) = (&self.client.root, &self.settings);
        let args = state.projects.get(root)?.build_args(&config, &None)?;

        let nvim = self.client.nvim(state)?;
        let ref mut logger = nvim.logger();

        logger.set_title(format!(
            "{}:{}",
            if is_once { "Build" } else { "Rebuild" },
            config.target
        ));

        tracing::trace!("building with [{}]", args.join(" "));
        let xclogger = XCLogger::new(&root, args)?;
        let success = logger.consume_build_logs(xclogger, false, is_once).await?;

        if !success {
            let ref msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(msg).await?;
        };

        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, _state: &MutexGuard<State>, event: &Event) -> bool {
        event.is_content_update_event()
            || event.is_rename_event()
            || event.is_create_event()
            || event.is_remove_event()
            || !(event.path().exists() || event.is_seen())
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _state: &MutexGuard<State>, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self, _state: &MutexGuard<State>) -> Result<()> {
        Ok(())
    }
}

impl std::fmt::Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:Build:{}", self.client.root.display(), self.settings)
    }
}
