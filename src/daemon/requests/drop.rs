use super::*;

/// Drop a client
#[derive(Debug, Serialize, Deserialize)]
pub struct Drop {
    client: Client,
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Drop {
    async fn handle(self, state: DaemonState) -> Result<()> {
        use tracing::*;
        trace!("{:?}", self);
        tokio::spawn(async move {
            let (root, pid) = (&self.client.root, self.client.pid);
            let mut state = state.lock().await;

            if let Err(e) = state.remove_workspace(&root, pid).await {
                error!("Unable to correctly drop client {e}")
            };

            state.validate(None).await?;

            anyhow::Ok(())
        });

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Drop> for Drop {
    fn pre(_lua: &Lua, _msg: &Drop) -> LuaResult<()> {
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Drop {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Drop"))
        }
    }
}
