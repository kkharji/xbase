use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;
use strum::{Display, EnumString};
#[cfg(feature = "daemon")]
use {
    crate::state::State,
    crate::watch::{Event, Watchable},
    crate::xcode::build_with_logger,
    tokio::sync::MutexGuard,
};

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
pub enum WatchOps {
    Stop,
    Start,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Display, EnumString)]
pub enum WatchKind {
    #[default]
    Build,
    Run,
}

/// TODO: Make it WatchBuildRequest
/// Watching a project target for build.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchTarget {
    pub client: Client,
    pub config: BuildConfiguration,
    pub ops: WatchOps,
    pub kind: WatchKind,
}

impl std::fmt::Display for WatchTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.client.root.display(),
            self.kind,
            self.config
        )
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for WatchTarget {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;

        let Self { client, ops, .. } = &self;

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;
        let watcher = client.get_watcher_mut(state)?;

        match ops {
            WatchOps::Start => watcher.add(self)?,
            WatchOps::Stop => watcher.remove(&self.to_string())?,
        };

        // TODO(daemon): update clients state account only for clients with specific project.
        // NOTE: Update all clients state
        state.sync_client_state().await?;

        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Watchable for WatchTarget {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        tracing::info!("Building {}", self.client.abbrev_root());
        let (root, config) = (&self.client.root, &self.config);
        let args = config.args(root, &None)?;

        let ref mut logger = self.client.nvim(state)?.logger();
        logger.set_title(format!("Rebuild:{}", config.target));
        build_with_logger(logger, root, &args, true, false).await?;

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

#[cfg(feature = "lua")]
impl<'a> Requester<'a, WatchTarget> for WatchTarget {
    fn pre(lua: &Lua, msg: &WatchTarget) -> LuaResult<()> {
        match msg.ops {
            WatchOps::Start => lua.print(&format!("{}", msg.config.to_string())),
            WatchOps::Stop => {
                lua.print(&format!("Stop watching with `{}`", msg.config.to_string()))
            }
        }

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for WatchTarget {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        use tap::Pipe;

        if let LuaValue::Table(table) = lua_value {
            let ops = if table.get::<_, String>("ops")? == "Start" {
                WatchOps::Start
            } else {
                WatchOps::Stop
            };

            let kind = table
                .get::<_, String>("kind")?
                .pipe(|s| WatchKind::from_str(&s).to_lua_err())
                .unwrap_or_default();

            let client = table.get("client")?;
            let config = table.get("config")?;

            Ok(Self {
                client,
                config,
                ops,
                kind,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Watch"))
        }
    }
}
