use {
    super::*, crate::nvim::BufferDirection, crate::store::DeviceLookup,
    crate::types::BuildConfiguration,
};

#[cfg(feature = "daemon")]
use {
    crate::constants::DAEMON_STATE, crate::run::RunService, crate::util::serde::value_or_default,
};

/// Run a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub client: Client,
    pub config: BuildConfiguration,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub device: DeviceLookup,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub direction: BufferDirection,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for RunRequest {
    async fn handle(self) -> Result<()> {
        tracing::info!("⚙️ Running: {}", self.config.to_string());

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        // TODO: Insert runner into state.runners
        RunService::new(state, self).await?;

        todo!();
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, RunRequest> for RunRequest {
    fn pre(lua: &Lua, msg: &RunRequest) -> LuaResult<()> {
        lua.print(&msg.to_string());
        Ok(())
    }
}

impl ToString for RunRequest {
    fn to_string(&self) -> String {
        if let Some(ref name) = self.device.name {
            format!("run [{}] with {}", name, self.config.to_string())
        } else {
            format!("run with {}", self.config.to_string())
        }
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for RunRequest {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        let table = match lua_value {
            LuaValue::Table(t) => Ok(t),
            _ => Err(LuaError::external("Fail to deserialize Run")),
        }?;

        let device: Option<LuaTable> = table.get("device")?;

        Ok(Self {
            client: table.get("client")?,
            config: table.get("config")?,
            direction: table.get("direction").unwrap_or_default(),
            device: device
                .map(|d| {
                    let name = d.get("name").ok()?;
                    let udid = d.get("udid").ok()?;
                    Some(DeviceLookup { name, udid })
                })
                .flatten()
                .unwrap_or_default(),
        })
    }
}
