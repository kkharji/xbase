use anyhow::Result;

use crate::Daemon;

#[derive(Debug)]
pub struct Build {
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

impl Build {
    pub fn new(_args: Vec<&str>) -> Result<Build> {
        Ok(Self {
            target: None,
            configuration: None,
            scheme: None,
        })
    }

    pub fn request(target: &str, configuration: &str, scheme: &str) -> Result<()> {
        Daemon::execute(&["build", target, configuration, scheme])
    }

    #[cfg(feature = "lua")]
    pub fn lua(
        lua: &mlua::Lua,
        (target, configuration, scheme): (String, String, String),
    ) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(&format!(
            "Build (target: {target} configuration: {configuration}, scheme: {scheme})"
        ))?;
        Self::request(&target, &configuration, &scheme).map_err(mlua::Error::external)
    }
}

#[async_trait::async_trait]
#[cfg(feature = "daemon")]
impl crate::DaemonCommandExt for Build {
    async fn handle(&self, _state: crate::state::SharedState) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}
