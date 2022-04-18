use anyhow::Result;

/// Register new client with workspace
#[derive(Debug)]
pub struct Drop {
    pub pid: i32,
    pub root: String,
}

impl Drop {
    pub fn new(args: Vec<&str>) -> Result<Self> {
        let pid = args.get(0);
        let root = args.get(1);

        if pid.is_none() || root.is_none() {
            anyhow::bail!("Missing arugments: [ pid: {:?}, root: {:?} ]", pid, root)
        }

        Ok(Self {
            pid: pid.unwrap().parse::<i32>()?,
            root: root.unwrap().to_string(),
        })
    }

    pub fn request(pid: i32, root: String) -> Result<()> {
        crate::Daemon::execute(&["drop", pid.to_string().as_str(), root.as_str()])
    }

    #[cfg(feature = "lua")]
    pub fn lua(lua: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(&format!("Dropped (pid: {pid} cwd: {root})"))?;
        Self::request(pid, root).map_err(mlua::Error::external)
    }
}

#[async_trait::async_trait]
#[cfg(feature = "daemon")]
impl crate::DaemonCommandExt for Drop {
    async fn handle(&self, state: crate::state::SharedState) -> Result<()> {
        tracing::trace!("{:?}", self);
        state
            .lock()
            .await
            .remove_workspace(&self.root, self.pid)
            .await
    }
}
