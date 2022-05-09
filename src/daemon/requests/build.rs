use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub client: Client,
    pub config: BuildConfiguration,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Build> for Build {
    fn pre(lua: &Lua, msg: &Build) -> LuaResult<()> {
        lua.print(&format!("{}", msg.config.to_string()));
        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Build {
    async fn handle(self, state: DaemonState) -> Result<()> {
        tracing::debug!("Handling build request..");
        use crate::xcode;

        let state = state.lock().await;
        let ws = state.get_workspace(&self.client.root)?;
        let nvim = ws.nvim(&self.client.pid)?;
        let stream = xcode::stream(&ws.root, vec!["build".to_string()], self.config).await?;
        nvim.log_to_buffer("Build", None, stream, false, true)
            .await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for Build {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                client: table.get("client")?,
                config: table.get("config")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Build"))
        }
    }
}
