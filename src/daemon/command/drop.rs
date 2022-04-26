use anyhow::Result;

/// Register new client with workspace
#[derive(Debug)]
pub struct Drop {
    pub pid: i32,
    pub root: String,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::daemon::DaemonCommandExt for Drop {
    async fn handle(&self, state: crate::daemon::DaemonState) -> Result<()> {
        tracing::trace!("{:?}", self);
        state
            .lock()
            .await
            .remove_workspace(&self.root, self.pid)
            .await
    }
}

impl TryFrom<Vec<&str>> for Drop {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        if let (Some(pid), Some(root)) = (args.get(0), args.get(1)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
            })
        } else {
            anyhow::bail!("Missing arugments: {:?}", args)
        }
    }
}

impl Drop {
    pub const KEY: &'static str = "drop";
    pub fn request(pid: i32, root: String) -> Result<()> {
        crate::daemon::Daemon::execute(&[Self::KEY, pid.to_string().as_str(), root.as_str()])
    }

    #[cfg(feature = "lua")]
    pub fn lua(_: &mlua::Lua, (pid, root): (i32, String)) -> mlua::Result<()> {
        Self::request(pid, root).map_err(mlua::Error::external)
    }
}
