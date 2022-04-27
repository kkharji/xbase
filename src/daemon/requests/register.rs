#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Register new client with workspace
#[derive(Debug)]
pub struct Register {
    pub pid: i32,
    pub root: String,
}

impl Register {
    pub const KEY: &'static str = "register";
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Register> for Register {
    fn parse(args: Vec<&str>) -> Result<Self> {
        if let (Some(pid), Some(root)) = (args.get(0), args.get(1)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
            })
        } else {
            anyhow::bail!("Missing arugments: got {:?}", args)
        }
    }
    async fn handle(&self, state: DaemonState) -> Result<()> {
        tracing::trace!("{:?}", self);
        state.lock().await.add_workspace(&self.root, self.pid).await
    }
}

#[cfg(feature = "lua")]
impl Register {
    pub fn lua(lua: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(&format!("Add (pid: {pid} cwd: {root})"))?;
        Self::request(pid, root).map_err(mlua::Error::external)
    }

    pub fn request(pid: i32, root: String) -> mlua::Result<()> {
        Daemon::execute(&[Self::KEY, pid.to_string().as_str(), root.as_str()])
    }
}
