use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;
use strum::{Display, EnumString};

#[derive(Clone, Debug, Serialize, Deserialize, Display, EnumString)]
pub enum WatchType {
    Build,
    Run,
}

/// Watch a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchStart {
    pub client: Client,
    pub watch_type: WatchType,
    pub config: BuildConfiguration,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, WatchStart> for WatchStart {
    fn pre(lua: &Lua, msg: &WatchStart) -> LuaResult<()> {
        lua.print(&format!("watching with {}", msg.config.to_string()));
        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for WatchStart {
    async fn handle(self, state: DaemonState) -> Result<()> {
        let current_state = state.clone();
        let config = self.config.to_string();
        let root = self.client.root.clone();
        tracing::debug!("Starting new watch service with {config}",);

        let mut current_state = current_state.lock().await;
        let ws = current_state.get_mut_workspace(&root)?;

        // Update state to indicate that a watch server is running
        for (_, nvim) in ws.clients.iter() {
            nvim.exec_lua("require'xcodebase.watch'.is_watching = true".into(), vec![])
                .await?;
        }

        ws.start_watch_service(self, crate::util::watch::new(root, state, event_handler))
            .await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for WatchStart {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::Table(table) = lua_value {
            let watch_type = table.get::<_, String>("watch_type")?;

            Ok(Self {
                watch_type: WatchType::from_str(&watch_type).to_lua_err()?,
                client: table.get("client")?,
                config: table.get("config")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Watch"))
        }
    }
}

#[cfg(feature = "daemon")]
pub async fn event_handler(
    state: DaemonState,
    root: String,
    path: std::path::PathBuf,
    event: notify::Event,
    last_seen: std::sync::Arc<tokio::sync::Mutex<String>>,
    debounce: std::sync::Arc<tokio::sync::Mutex<std::time::SystemTime>>,
) -> Result<bool, crate::util::watch::WatchError> {
    use crate::util::watch::{self, WatchError};
    use std::time::Duration;

    if !(matches!(
        event.kind,
        notify::EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Content
        ))
    ) || matches!(event.kind, notify::EventKind::Create(_))
        || matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
        ))
    {
        return Ok(false);
    }

    if let notify::EventKind::Modify(notify::event::ModifyKind::Name(_)) = &event.kind {
        // HACK: only account for new path and skip duplications
        let path_string = path.to_string_lossy();
        if !path.exists() || watch::should_ignore(last_seen.clone(), &path_string).await {
            return Ok(false);
        }
        tokio::time::sleep(Duration::new(1, 0)).await;
    }

    tracing::debug!("Rebuilding for {:#?}", &event);

    let state = state.lock().await;
    let ws = state
        .get_workspace(&root)
        .map_err(|e| WatchError::Stop(e.to_string()))?;

    let (watch_req, _) = ws
        .watch
        .as_ref()
        .ok_or_else(|| WatchError::Stop("No watch handle, breaking".into()))?;

    let nvim = ws
        .get_client(&watch_req.client.pid)
        .map_err(|e| WatchError::Stop(e.to_string()))?;

    let stream = match watch_req.watch_type {
        WatchType::Build => ws
            .project
            .xcodebuild(&["build"], watch_req.config.clone())
            .await
            .map_err(|e| WatchError::Continue(format!("Build Failed: {e}")))?,

        WatchType::Run => {
            nvim.log_error("Watch", "Run is not supported yet! .. aborting")
                .await
                .map_err(|e| WatchError::Stop(format!("Unable to log to nvim buffer: {e}")))?;

            nvim.exec_lua(
                "require'xcodebase.watch'.is_watching = false".into(),
                vec![],
            )
            .await
            .map_err(|e| WatchError::Stop(format!("Unable to log to nvim buffer: {e}")))?;

            return Err(WatchError::Stop("Run not supported yet!".into()));
        }
    };
    // TODO(nvim): Ensure that exiting a nvim instance will stop build watcher
    nvim.log_to_buffer("Watch", None, stream, false, false)
        .await
        .map_err(|e| WatchError::Continue(format!("Logging to client failed: {e}")))?;

    let mut debounce = debounce.lock().await;
    *debounce = std::time::SystemTime::now();

    Ok(true)
}
