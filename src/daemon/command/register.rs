use crate::state::SharedState;
use crate::{Daemon, DaemonCommandExt};
use anyhow::{bail, Result};
use async_trait::async_trait;
use tracing::trace;

/// Register new client with workspace
#[derive(Debug)]
pub struct Register {
    pub pid: i32,
    pub root: String,
}

impl Register {
    pub fn new(args: Vec<&str>) -> Result<Self> {
        let pid = args.get(0);
        let root = args.get(1);

        if pid.is_none() || root.is_none() {
            bail!("Missing arugments: [ pid: {:?}, root: {:?} ]", pid, root)
        }

        Ok(Self {
            pid: pid.unwrap().parse::<i32>()?,
            root: root.unwrap().to_string(),
        })
    }

    pub fn request(pid: i32, root: String) -> Result<()> {
        Daemon::execute(&["register", pid.to_string().as_str(), root.as_str()])
    }

    #[cfg(feature = "lua")]
    pub fn lua(lua: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(&format!("Removed (pid: {pid} cwd: {root})"))?;
        Self::request(pid, root).map_err(mlua::Error::external)
    }
}

#[async_trait]
impl DaemonCommandExt for Register {
    async fn handle(&self, state: SharedState) -> Result<()> {
        trace!("{:?}", self);
        state.lock().await.add_workspace(&self.root, self.pid).await
    }
}
