#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Run a project.
#[derive(Debug)]
pub struct Run {
    _simulator: bool,
}

impl Run {
    pub const KEY: &'static str = "run";
}

// TODO: Implement run command
//
// Also, it might be important to pick which target/paltform to run under. This is currently just
// either with a simulator or not assuming only the use case won't include
// macos apps, which is wrong
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Run> for Run {
    fn parse(args: Vec<&str>) -> Result<Self> {
        let _simulator = args.get(0).unwrap_or(&"").parse::<bool>().unwrap_or(false);
        Ok(Self { _simulator })
    }

    async fn handle(&self, _state: DaemonState) -> Result<()> {
        tracing::info!("Run command");
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl Run {
    pub fn lua(lua: &mlua::Lua, with_simulator: bool) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(&format!("Run command called"))?;
        Daemon::execute(&[Self::KEY, &with_simulator.to_string()])
    }
}
