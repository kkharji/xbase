use super::*;
use crate::{nvim::BufferDirection, types::BuildConfiguration};
use std::fmt::Debug;

#[cfg(feature = "daemon")]
use {
    crate::constants::DAEMON_STATE, crate::util::serde::value_or_default,
    crate::xcode::build_with_logger,
};

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub client: Client,
    pub config: BuildConfiguration,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub direction: BufferDirection,
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Build {
    async fn handle(self) -> Result<()> {
        let Self { client, config, .. } = &self;
        let Client { root, .. } = client;

        tracing::debug!("Handling build request {:#?}", config);

        let state = DAEMON_STATE.clone();
        let ref state = state.lock().await;

        let nvim = client.nvim(state)?;
        let direction = self.direction.clone();
        let args = config.args(&root, &None)?;

        let ref mut logger = nvim.logger();

        logger.set_title(format!("Build:{}", config.target));
        logger.set_direction(&direction);

        let success = build_with_logger(logger, root, &args, true, true).await?;

        if !success {
            let ref msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(msg).await?;
        };

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
            direction: table.get("direction").unwrap_or_default(),
        })
    }
}
