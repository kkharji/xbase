use crate::types::Client;

use super::*;

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

        let Register { client, .. } = &self;
        let Client { root, pid } = &client;

        tracing::info!("Register({pid}, {}): ", client.abbrev_root());

        let state = DAEMON_STATE.clone();
        let mut state = state.lock().await;

        if let Ok(project) = state.projects.get_mut(root) {
            // NOTE: Add client pid to project
            project.clients.push(*pid);
        } else {
            // NOTE: Create nvim client
            state.projects.add(&self).await?;

            let project = state.projects.get(root).unwrap();
            let ignore_patterns = project.ignore_patterns.clone();

            // NOTE: Add watcher
            state
                .watcher
                .add_project_watcher(client, ignore_patterns)
                .await
        }

        // NOTE: Create nvim client
        state.clients.add(&self).await?;

        // NOTE: Sink Daemon to nvim vim.g.xbase.state
        let _update_handle = state.sync_client_state().await?;

        // TODO(register): Ensure buildServer.json and .compile exists in a subprocess

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Register {
    fn from_lua(v: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = v {
            Ok(Self {
                address: table.get("address")?,
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Register"))
        }
    }
}
