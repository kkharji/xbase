use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;
use strum::{Display, EnumString};

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

/// Stop Watching a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchTarget {
    pub client: Client,
    pub config: BuildConfiguration,
    pub ops: WatchOps,
    pub kind: WatchKind,
}

impl WatchTarget {
    pub fn key(&self) -> String {
        format!(
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

        let Self {
            client,
            config,
            ops,
            ..
        } = &self;

        let BuildConfiguration { target, .. } = config;

        let state = DAEMON_STATE.clone();
        let mut state = state.lock().await;

        if target.is_empty() {
            anyhow::bail!("No target specified!")
        }

        match ops {
            WatchOps::Start => {
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
            WatchOps::Stop => {
                // NOTE: Remove target watcher
                state.watcher.remove_target_watcher(&self).await;
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
            WatchOps::Start => {
                lua.print(&format!("{}", msg.config.to_string()));
            }
            WatchOps::Stop => {
                lua.print(&format!("Stop watching with `{}`", msg.config.to_string()));
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
