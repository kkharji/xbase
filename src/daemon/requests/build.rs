use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub client: Client,
    pub config: BuildConfiguration,
    pub direction: Option<BufferDirection>,
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
        use crate::nvim::BulkLogRequest;
        use crate::xcode;

        let state = state.lock().await;
        let ws = state.get_workspace(&self.client.root)?;
        let nvim = ws.nvim(&self.client.pid)?;

        nvim.buffers
            .log
            .bulk_append(BulkLogRequest {
                nvim,
                title: "Build",
                direction: self.direction,
                stream: xcode::stream(&ws.root, vec!["build".to_string()], self.config).await?,
                clear: false,
                open: true,
            })
            .await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for Build {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::Table(table) = lua_value {
            let mut direction = None;
            if let Some(str) = table.get::<_, Option<String>>("direction")? {
                direction = BufferDirection::from_str(&str).ok();
            }
            Ok(Self {
                client: table.get("client")?,
                config: table.get("config")?,
                direction,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Build"))
        }
    }
}
