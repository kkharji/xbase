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
    pub address: String,
    pub root: String,
}

impl Register {
    pub const KEY: &'static str = "register";
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Register> for Register {
    fn parse(args: Vec<&str>) -> Result<Self> {
        if let (Some(pid), Some(root), Some(address)) = (args.get(0), args.get(1), args.get(2)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
                address: address.to_string(),
            })
        } else {
            anyhow::bail!("Missing arguments: got {:?}", args)
        }
    }

    async fn handle(&self, state: DaemonState) -> Result<()> {
        tracing::trace!("{:?}", self);
        state
            .lock()
            .await
            .add_workspace(&self.root, self.pid, &self.address)
            .await
    }
}

#[cfg(feature = "lua")]
impl Register {
    pub fn lua(lua: &mlua::Lua, (pid, root, address): (i32, String, String)) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        Daemon::execute(&[Self::KEY, &pid.to_string(), &root, &address])?;
        lua.trace(&format!("registered client (pid: {pid})"))?;
        Ok(())
    }
}
