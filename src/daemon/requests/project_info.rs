use super::*;

/// Return current working direcotry project info
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub client: Client,
}

#[cfg(feature = "lua")]
impl<'a> Requestor<'a, ProjectInfo> for ProjectInfo {}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for ProjectInfo {
    async fn handle(&self, state: DaemonState) -> Result<()> {
        let (root, pid) = (&self.client.root, self.client.pid);
        tracing::info!("Getting info for {}", root);

        let state = state.lock().await;
        let workspace = state.get_workspace(root)?;
        let nvim = workspace.get_client(&pid)?;
        let project = serde_json::to_string(&workspace.project)?;
        let script = format!(
            "require'xcodebase.state'.projects['{root}'] = vim.json.decode([[{project}]])",
        );

        nvim.exec_lua(&script, vec![]).await?;

        Ok(())
    }
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
