#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::{bail, Result};

/// Return curr working direcotry project info
#[derive(Debug)]
pub struct ProjectInfo {
    pub pid: i32,
    pub root: String,
}

impl ProjectInfo {
    pub const KEY: &'static str = "project_info";
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<ProjectInfo> for ProjectInfo {
    fn parse(args: Vec<&str>) -> Result<Self> {
        if let (Some(pid), Some(root)) = (args.get(0), args.get(1)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
            })
        } else {
            anyhow::bail!("Missing arugments: {:?}", args)
        }
    }

    async fn handle(&self, state: DaemonState) -> Result<()> {
        tracing::info!("Getting info for {}", self.root);
        let state = state.lock().await;

        let workspace = match state.workspaces.get(&self.root) {
            Some(o) => o,
            None => bail!("No workspace for {}", self.root),
        };

        let nvim = match workspace.clients.get(&self.pid) {
            Some(o) => o,
            None => bail!("No nvim instance for {}", self.pid),
        };

        nvim.exec_lua(
            &format!(
                "require'xcodebase.state'.projects['{}'] = vim.json.decode([[{}]])",
                self.root,
                serde_json::to_string(&workspace.project)?
            ),
            vec![],
        )
        .await?;

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl ProjectInfo {
    pub fn lua(lua: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(&format!("grapping project info"))?;
        Daemon::execute(&[Self::KEY, pid.to_string().as_str(), root.as_str()])
    }
}
