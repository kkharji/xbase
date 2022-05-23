use super::*;

/// Register new client with workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub client: Client,
}

#[cfg(feature = "daemon")]
use crate::constants::DAEMON_STATE;

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Register {
    async fn handle(self) -> Result<()> {
        let Self { client } = &self;
        let Client { root, .. } = &client;
        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        client.register_self(state).await?;
        client.register_project(state).await?;
        if client.ensure_server_support(state, None).await? {
            let ref name = client.abbrev_root();
            state.clients.echo_msg(root, name, "setup: ✅").await;
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
