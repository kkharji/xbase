use anyhow::Result;

/// Register new client with workspace
#[derive(Debug)]
pub struct Register {
    pub pid: i32,
    pub root: String,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::DaemonCommandExt for Register {
    async fn handle(&self, state: crate::SharedState) -> Result<()> {
        tracing::trace!("{:?}", self);
        state.lock().await.add_workspace(&self.root, self.pid).await
    }
}

impl TryFrom<Vec<&str>> for Register {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        if let (Some(pid), Some(root)) = (args.get(0), args.get(1)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
            })
        } else {
            anyhow::bail!("Missing arugments: got {:?}", args)
        }
    }
}

impl Register {
    pub const KEY: &'static str = "register";
    pub fn request(pid: i32, root: String) -> Result<()> {
        crate::Daemon::execute(&[Self::KEY, pid.to_string().as_str(), root.as_str()])
    }
}

#[cfg(feature = "lua")]
impl Register {
    pub fn lua(lua: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(&format!("Add (pid: {pid} cwd: {root})"))?;
        Self::request(pid, root).map_err(mlua::Error::external)
    }
}
