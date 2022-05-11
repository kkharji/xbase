use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;

/// Watch a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchStart {
    pub client: Client,
    pub request: BuildConfiguration,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, WatchStart> for WatchStart {
    fn pre(lua: &Lua, msg: &WatchStart) -> LuaResult<()> {
        lua.print(&format!("watching with {}", msg.request.to_string()));
        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for WatchStart {
    async fn handle(self, state: DaemonState) -> Result<()> {
        if self.request.target.is_empty() {
            anyhow::bail!("No target specified!")
        }
        let current_state = state.clone();
        let root = self.client.root.clone();
        tracing::debug!(
            "Starting new watch service with {}",
            self.request.to_string()
        );

        tracing::debug!("Starting new watch service with {:#?}", self);
        current_state
            .lock()
            .await
            .watch(&root, Some(self), state)
            .await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for WatchStart {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                client: table.get("client")?,
                request: table.get("request")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Watch"))
        }
    }
}
