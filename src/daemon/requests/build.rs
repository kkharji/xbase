use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

#[cfg(feature = "daemon")]
use {
    crate::constants::DAEMON_STATE,
    crate::state::State,
    crate::util::serde::value_or_default,
    crate::watch::{Event, Watchable},
    crate::xcode::build_with_logger,
    tokio::sync::MutexGuard,
};

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub client: Client,
    pub config: BuildConfiguration,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub direction: BufferDirection,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub ops: RequestOps,
}

#[cfg(feature = "daemon")]
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

#[cfg(feature = "daemon")]
#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        tracing::info!("Building {}", self.client.abbrev_root());
        let (root, config) = (&self.client.root, &self.config);
        let args = config.args(root, &None)?;

        let nvim = self.client.nvim(state)?;
        let ref mut logger = nvim.logger();

        logger.set_title(format!("Rebuild:{}", config.target));
        let success = build_with_logger(logger, root, &args, true, self.ops.is_once()).await?;
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
        write!(f, "{}:Build:{}", self.client.root.display(), self.config)
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, BuildRequest> for BuildRequest {
    fn pre(lua: &Lua, msg: &BuildRequest) -> LuaResult<()> {
        match msg.ops {
            RequestOps::Watch => lua.print(&format!("Watch {}", msg.config.to_string())),
            RequestOps::Stop => lua.print(&format!("Stop {}", msg.config.to_string())),
            RequestOps::Once => lua.print(&format!("{}", msg.config.to_string())),
        }

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for BuildRequest {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        let table = match lua_value {
            LuaValue::Table(table) => table,
            _ => return Err(LuaError::external("Fail to deserialize Build")),
        };

        Ok(Self {
            client: table.get("client")?,
            config: table.get("config")?,
            ops: table.get("ops").unwrap_or_default(),
            direction: table.get("direction").unwrap_or_default(),
        })
    }
}
