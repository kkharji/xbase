use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

#[cfg(feature = "daemon")]
use crate::{constants::DAEMON_STATE, nvim::Logger, xcode::stream_build};

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

        let state = DAEMON_STATE.lock().await;
        let nvim = state
            .clients
            .get(pid)
            .ok_or_else(|| anyhow::anyhow!("no client found with {}", self.client.pid))?;

        let direction = self.direction.clone();

        Logger::new(nvim, "Build", &config)
            .log_stream(stream_build(&root, &config).await?, direction, true, true)
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
        use std::str::FromStr;
        let mut direction = None;

        let table = match lua_value {
            LuaValue::Table(table) => table,
            _ => return Err(LuaError::external("Fail to deserialize Build")),
        };

        if let Some(str) = table.get::<_, Option<String>>("direction")? {
            direction = BufferDirection::from_str(&str).ok();
        }

        Ok(Self {
            client: table.get("client")?,
            config: table.get("config")?,
            direction,
        })
    }
}
