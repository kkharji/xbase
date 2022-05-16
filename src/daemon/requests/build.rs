use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

#[cfg(feature = "daemon")]
use crate::{constants::DAEMON_STATE, xcode::stream_build};

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub client: Client,
    pub config: BuildConfiguration,
    pub direction: Option<BufferDirection>,
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Build {
    async fn handle(self) -> Result<()> {
        let Self { client, config, .. } = &self;
        let Client { pid, root } = client;

        tracing::debug!("Handling build request {:#?}", self.config);

        let state = DAEMON_STATE.clone().lock_owned().await;
        let nvim = state
            .clients
            .get(pid)
            .ok_or_else(|| anyhow::anyhow!("no client found with {}", self.client.pid))?;

        let direction = self.direction.clone();

        nvim.new_logger("build", config, &direction)
            .log_stream(stream_build(&root, &config).await?, true, true)
            .await?;

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Build> for Build {
    fn pre(lua: &Lua, msg: &Build) -> LuaResult<()> {
        lua.print(&format!("{}", msg.config.to_string()));
        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for Build {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        let table = match lua_value {
            LuaValue::Table(table) => table,
            _ => return Err(LuaError::external("Fail to deserialize Build")),
        };

        Ok(Self {
            client: table.get("client")?,
            config: table.get("config")?,
            direction: table.get("direction").ok(),
        })
    }
}
