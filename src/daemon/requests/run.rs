use super::*;

/// Run a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    simulator: bool,
    client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Run> for Run {}

// TODO: Implement run command
//
// Also, it might be important to pick which target/paltform to run under. This is currently just
// either with a simulator or not assuming only the use case won't include
// macos apps, which is wrong
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for Run {
    async fn handle(self, _state: DaemonState) -> Result<()> {
        tracing::info!("Run command");
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Run {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                simulator: table.get("simulator")?,
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Run"))
        }
    }
}
