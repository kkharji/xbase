use super::*;

/// Drop a client
#[derive(Debug, Serialize, Deserialize)]
pub struct Drop {
    client: Client,
    #[serde(default)]
    remove_client: bool,
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Drop {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;
        let Self {
            client,
            remove_client,
        } = self;

        let state = DAEMON_STATE.clone();
        let mut state = state.lock().await;

        if state.clients.contains_key(&client.pid) {
            tracing::info!("Drop({}: {})", client.pid, client.abbrev_root());
            // NOTE: Should only be Some if no more client depend on it
            if let Some(project) = state.projects.remove(&client).await? {
                // NOTE: Remove project watchers
                state.watcher.remove_project_watcher(&client).await;
                // NOTE: Remove target watchers associsated with root
                state
                    .watcher
                    .remove_target_watcher_for_root(&project.root)
                    .await;
            }
            // NOTE: Try removing client with given pid
            if remove_client {
                tracing::debug!("RemoveClient({})", client.pid);
                state.clients.remove(&client.pid);
            }
            // NOTE: Sink state to all client vim.g.xbase.state
            state.sync_client_state().await?;
        }

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
                remove_client: match table.get::<_, bool>("remove_client") {
                    Ok(value) => value,
                    Err(_) => true,
                },
            })
        } else {
            Err(LuaError::external("Fail to deserialize Drop"))
        }
    }
}
