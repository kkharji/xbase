use super::*;
use crate::types::Client;

/// Register new client with workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub address: String,
    pub client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Register> for Register {
    fn pre(lua: &Lua, msg: &Register) -> LuaResult<()> {
        lua.trace(&format!("Registered client ({})", msg.client.pid))
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Register {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;
        use tap::Pipe;

        // NOTE: This logic should maybe moved to tokio::spawn to speed up initialization.
        let name = self.client.abbrev_root();
        tracing::info!("Register({}, {}): ", self.client.pid, name);

        let mut state = DAEMON_STATE.clone().lock_owned().await;

        if let Ok(project) = state.projects.get_mut(&self.client.root) {
            // NOTE: Add client pid to project
            project.clients.push(self.client.pid.clone());
        } else {
            // NOTE: Create nvim client
            state.projects.add(&self).await?;

            let project = state.projects.get(&self.client.root).unwrap();
            let ignore_patterns = project.ignore_patterns.clone();

            // NOTE: Add watcher
            state
                .watcher
                .add_project_watcher(&self.client, ignore_patterns)
                .await
        }

        // NOTE: Create nvim client
        state.clients.add(&self).await?;

        tokio::spawn(async move {
            let ref mut state = DAEMON_STATE.clone().lock_owned().await;

            // NOTE: Ensure project is ready for xbase build server
            let generated =
                crate::compile::ensure_server_support(state, &name, &self.client.root, None)
                    .await?;

            if generated {
                "setup: ✅"
                    .pipe(|msg| state.clients.echo_msg(&self.client.root, &name, msg))
                    .await;
            }

            // NOTE: Sink Daemon to nvim vim.g.xbase
            state.sync_client_state().await?;
            anyhow::Ok(())
        });

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Register {
    fn from_lua(v: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = v {
            Ok(Self {
                address: match table.get("address") {
                    Ok(v) => v,
                    Err(_) => return Err(LuaError::external("Unable to get client address!")),
                },
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Register"))
        }
    }
}
