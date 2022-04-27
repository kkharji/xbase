#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Drop a client
#[derive(Debug)]
pub struct Drop {
    pub pid: i32,
    pub root: String,
}

impl Drop {
    pub const KEY: &'static str = "drop";
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Drop> for Drop {
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
        tracing::trace!("{:?}", self);
        state
            .lock()
            .await
            .remove_workspace(&self.root, self.pid)
            .await
    }
}

#[cfg(feature = "lua")]
impl Drop {
    pub fn lua(_: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        Self::request(pid, root).map_err(mlua::Error::external)
    }

    pub fn request(pid: i32, root: String) -> mlua::Result<()> {
        Daemon::execute(&[Self::KEY, pid.to_string().as_str(), root.as_str()])
    }
}
