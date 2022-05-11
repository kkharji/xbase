use super::*;
use std::fmt::Debug;

/// Stop Watching a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchStop {
    pub client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, WatchStop> for WatchStop {
    fn pre(lua: &Lua, _msg: &WatchStop) -> LuaResult<()> {
        lua.print(&format!("Stopping watching service .."));
        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for WatchStop {
    async fn handle(self, state: DaemonState) -> Result<()> {
        let root = self.client.root.clone();
        let mut state = state.lock().await;
        state.validate(Some(self.client)).await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for WatchStop {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize WatchStop"))
        }
    }
}
