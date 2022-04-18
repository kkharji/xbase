use anyhow::Result;

use crate::Daemon;

#[derive(Debug)]
pub struct Build {
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

// TODO: Implement build command
// On neovim side:
// - Call the command after picking the target. If their is only a single target then just use that
//  - This requires somehow given the client all information it needs in order present the user
//  with the options needed to build
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::DaemonCommandExt for Build {
    async fn handle(&self, _state: crate::state::SharedState) -> Result<()> {
        tracing::info!("build command");
        Ok(())
    }
}

impl TryFrom<Vec<&str>> for Build {
    type Error = anyhow::Error;

    fn try_from(_args: Vec<&str>) -> Result<Self, Self::Error> {
        Ok(Self {
            target: None,
            configuration: None,
            scheme: None,
        })
    }
}

impl Build {
    pub const KEY: &'static str = "build";

    pub fn request(target: &str, configuration: &str, scheme: &str) -> Result<()> {
        Daemon::execute(&["build", target, configuration, scheme])
    }
}

impl Build {
    #[cfg(feature = "lua")]
    pub fn lua(lua: &mlua::Lua, (t, c, s): (String, String, String)) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(format!("Build (target: {t} configuration: {c}, scheme: {s})").as_ref())?;
        Self::request(&t, &c, &s).map_err(mlua::Error::external)
    }
}
