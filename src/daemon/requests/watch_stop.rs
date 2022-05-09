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
        let current_state = state.clone();
        let root = self.client.root.clone();
        let mut current_state = current_state.lock().await;
        let ws = current_state.get_mut_workspace(&root)?;

        ws.stop_watch_service().await?;

        // Update state to indicate that a watch server is disabled
        for (_, nvim) in ws.clients.iter() {
            nvim.exec_lua(
                "require'xcodebase.watch'.is_watching = false".into(),
                vec![],
            )
            .await?;
        }

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
