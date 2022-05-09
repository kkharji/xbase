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
        tracing::trace!("{:?}", self);
        let (root, pid) = (&self.client.root, self.client.pid);
        let watch_state = state.clone();
        let watch_root = root.to_string();

        // Drop watch service for current client
        tokio::spawn(async move {
            let mut state = watch_state.lock().await;
            let ws = state.get_mut_workspace(&watch_root)?;

            if ws.is_watch_service_running() {
                let mut stop = false;
                if let Some((watch_req, _)) = ws.watch.as_ref() {
                    if watch_req.client.pid == pid {
                        stop = true;
                    }
                    if stop {
                        ws.stop_watch_service().await?;
                    }
                }
            }
            anyhow::Ok(())
        });

        let mut state = state.lock().await;
        state.remove_workspace(root, pid).await
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
