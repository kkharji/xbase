use {
    super::*, crate::constants::DAEMON_STATE, crate::nvim::BufferDirection, crate::run::RunService,
    crate::store::DeviceLookup, crate::types::BuildConfiguration,
    crate::util::serde::value_or_default,
};

/// Run a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub client: Client,
    pub settings: BuildConfiguration,
    #[serde(deserialize_with = "value_or_default")]
    pub device: DeviceLookup,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    #[serde(deserialize_with = "value_or_default")]
    pub ops: RequestOps,
}

#[async_trait::async_trait]
impl Handler for RunRequest {
    async fn handle(self) -> Result<()> {
        let ref key = self.to_string();
        tracing::info!("⚙️ Running: {}", self.settings.to_string());

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

impl std::fmt::Display for RunRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:Run:{}:{}",
            self.client.root.display(),
            self.device.name.as_ref().unwrap_or(&"Bin".to_string()),
            self.settings
        )
    }
}
