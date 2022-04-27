#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Build a project.
#[derive(Debug)]
pub struct Build {
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

impl Build {
    pub const KEY: &'static str = "build";
}

// TODO: Implement build command
// On neovim side:
// - Call the command after picking the target. If their is only a single target then just use that
//  - This requires somehow given the client all information it needs in order present the user
//  with the options needed to build
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Build> for Build {
    fn parse(_args: Vec<&str>) -> Result<Self> {
        Ok(Self {
            target: None,
            configuration: None,
            scheme: None,
        })
    }

    async fn handle(&self, _state: DaemonState) -> Result<()> {
        tracing::info!("build command");
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl Build {
    pub fn lua(lua: &mlua::Lua, (t, c, s): (String, String, String)) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(format!("Build (target: {t} configuration: {c}, scheme: {s})").as_ref())?;
        Self::request(&t, &c, &s).map_err(mlua::Error::external)
    }

    pub fn request(target: &str, configuration: &str, scheme: &str) -> mlua::Result<()> {
        Daemon::execute(&["build", target, configuration, scheme])
    }
}
