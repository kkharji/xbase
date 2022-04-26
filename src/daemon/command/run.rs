use anyhow::Result;

/// Action to Run a project.
#[derive(Debug)]
pub struct Run {
    _simulator: bool,
}

// TODO: Implement run command
//
// Also, it might be important to pick which target/paltform to run under. This is currently just
// either with a simulator or not assuming only the use case won't include
// macos apps, which is wrong
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::daemon::DaemonCommandExt for Run {
    async fn handle(&self, _state: crate::daemon::DaemonState) -> Result<()> {
        tracing::info!("Run command");
        Ok(())
    }
}

impl TryFrom<Vec<&str>> for Run {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        let _simulator = args.get(0).unwrap_or(&"").parse::<bool>().unwrap_or(false);
        Ok(Self { _simulator })
    }
}

impl Run {
    pub const KEY: &'static str = "run";

    pub fn request(with_simulator: bool) -> Result<()> {
        crate::daemon::Daemon::execute(&[Self::KEY, &with_simulator.to_string()])
    }
}

#[cfg(feature = "lua")]
impl Run {
    pub fn lua(lua: &mlua::Lua, with_simulator: bool) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(&format!("Run command called"))?;
        Self::request(with_simulator).map_err(mlua::Error::external)
    }
}
