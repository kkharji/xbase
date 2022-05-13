use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WatchKind {
    Stop,
    Start,
}

/// Stop Watching a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchTarget {
    pub client: Client,
    pub config: BuildConfiguration,
    pub ops: WatchKind,
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for WatchTarget {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;

        let Self {
            client,
            config,
            ops,
        } = &self;

        let BuildConfiguration { target, .. } = config;

        let state = DAEMON_STATE.clone();
        let mut state = state.lock().await;

        if target.is_empty() {
            anyhow::bail!("No target specified!")
        }

        match ops {
            WatchKind::Start => {
                // NOTE: Get project associate ignore pattern
                let ignore_patterns = state
                    .projects
                    .get_mut(&client.root)
                    .ok_or_else(|| anyhow::anyhow!("No project for {:#?}", config))?
                    .ignore_patterns
                    .clone();

                // NOTE: add new target watcher
                state
                    .watcher
                    .add_target_watcher(&self, ignore_patterns)
                    .await;
            }
            WatchKind::Stop => {
                // NOTE: Remove target watcher
                state.watcher.remove_target_watcher(&self, client).await;
            }
        }

        // TODO(daemon): update clients state account only for clients with specific project.
        // NOTE: Update all clients state
        state.sync_client_state().await?;

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, WatchTarget> for WatchTarget {
    fn pre(lua: &Lua, msg: &WatchTarget) -> LuaResult<()> {
        match msg.ops {
            WatchKind::Start => {
                lua.print(&format!("{}", msg.config.to_string()));
            }
            WatchKind::Stop => {
                lua.print(&format!("Stopping watching service .."));
            }
        }

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for WatchTarget {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            let ops = if table.get::<_, String>("ops")? == "Start" {
                WatchKind::Start
            } else {
                WatchKind::Stop
            };

            Ok(Self {
                client: table.get("client")?,
                config: table.get("config")?,
                ops,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Watch"))
        }
    }
}
