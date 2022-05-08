use super::*;

/// Register new client with workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub address: String,
    pub client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requestor<'a, Register> for Register {
    fn pre(lua: &Lua, msg: &Register) -> LuaResult<()> {
        lua.trace(&format!("Registered client ({})", msg.client.pid))
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Register {
    async fn handle(&self, state: DaemonState) -> anyhow::Result<()> {
        tracing::trace!("{:?}", self);
        let mut state = state.lock().await;
        let client = &self.client;

        state
            .add_workspace(&client.root, client.pid, &self.address)
            .await
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
