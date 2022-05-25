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
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub ops: RequestOps,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for RunRequest {
    async fn handle(self) -> Result<()> {
        let ref key = self.to_string();
        tracing::info!("⚙️ Running: {}", self.config.to_string());

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if self.ops.is_once() {
            // TODO(run): might want to keep track of ran services
            RunService::new(state, self).await?;
            return Ok(());
        }

        let client = self.client.clone();
        if self.ops.is_watch() {
            let watcher = client.get_watcher(state)?;
            if watcher.contains_key(key) {
                client
                    .nvim(state)?
                    .echo_err("Already watching with {key}!!")
                    .await?;
            } else {
                let run_service = RunService::new(state, self).await?;
                let watcher = client.get_watcher_mut(state)?;
                watcher.add(run_service)?;
            }
        } else {
            let watcher = client.get_watcher_mut(state)?;
            let listener = watcher.remove(&self.to_string())?;
            listener.discard(state).await?;
        }

        state.sync_client_state().await?;
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, RunRequest> for RunRequest {
    fn pre(lua: &Lua, msg: &RunRequest) -> LuaResult<()> {
        lua.print(&msg.to_string());
        Ok(())
    }
}

impl std::fmt::Display for RunRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:Run:{}:{}",
            self.client.root.display(),
            self.device.name.as_ref().unwrap_or(&"Bin".to_string()),
            self.config
        )
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
            ops: table.get("ops").unwrap_or_default(),
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
