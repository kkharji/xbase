use super::*;

/// Register new client with workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub client: crate::types::Client,
}

#[cfg(feature = "daemon")]
use {
    crate::{compile::ensure_server_support, constants::DAEMON_STATE},
    tap::Pipe,
};

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Register {
    async fn handle(self) -> Result<()> {
        let Self { client } = &self;
        let Client { pid, root, .. } = &client;
        let name = client.abbrev_root();
        let ref mut state = DAEMON_STATE.clone().lock_owned().await;

        tracing::info!("Register({pid}, {name}): ");

        // NOTE: Create nvim client
        state.clients.add(client).await?;

        if let Ok(project) = state.projects.get_mut(root) {
            // NOTE: Add client pid to project
            project.clients.push(*pid);
        } else {
            // NOTE: Create nvim client
            state.projects.add(client).await?;

            let project = state.projects.get(root).unwrap();
            let ignore_patterns = project.ignore_patterns.clone();

            // NOTE: Add watcher
            state
                .watcher
                .add_project_watcher(client, ignore_patterns)
                .await
        }

        // NOTE: Ensure project is ready for xbase build server
        if ensure_server_support(state, &name, &self.client.root, None).await? {
            "setup: ✅"
                .pipe(|msg| state.clients.echo_msg(&self.client.root, &name, msg))
                .await;
        }

        // NOTE: Sink Daemon to nvim vim.g.xbase
        state.sync_client_state().await?;

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Register {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        Client::from_lua(value, lua).map(|client| Self { client })
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Register> for Register {
    fn pre(lua: &Lua, msg: &Register) -> LuaResult<()> {
        lua.trace(&format!("Registered client ({})", msg.client.pid))
    }
}
