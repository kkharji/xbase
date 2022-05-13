// TODO(daemon): Remove ProjectInfo Request
//
// This is no longer relevant as the internal state get validated automatically
use super::*;

/// Return current working direcotry project info
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, ProjectInfo> for ProjectInfo {}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for ProjectInfo {
    // async fn handle(self, state: DaemonState) -> Result<()> {
    //     let (root, _) = (&self.client.root, self.client.pid);
    //     tracing::info!("Getting info for {}", root);

    //     let mut state = state.lock().await;
    //     state.get_mut_workspace(root)?.sync_state().await?;

    //     Ok(())
    // }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for ProjectInfo {
    fn from_lua(v: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = v {
            Ok(Self {
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize ProjectInfo"))
        }
    }
}
