use anyhow::Result;

/// Rename file + class
#[derive(Debug)]
pub struct Run {
    _simulator: bool,
}

impl Run {
    pub fn new(args: Vec<&str>) -> Result<Self> {
        let _simulator = args.get(0).unwrap_or(&"").parse::<bool>().unwrap_or(false);
        Ok(Self { _simulator })
    }

    pub fn request(path: &str, name: &str, new_name: &str) -> Result<()> {
        crate::Daemon::execute(&["run", path, name, new_name])
    }

    #[cfg(feature = "lua")]
    pub fn lua(
        lua: &mlua::Lua,
        (path, name, new_name): (String, String, String),
    ) -> mlua::Result<()> {
        use crate::LuaExtension;
        lua.trace(&format!("Run command called"))?;
        Self::request(&path, &name, &new_name).map_err(mlua::Error::external)
    }
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::DaemonCommandExt for Run {
    async fn handle(&self, _state: crate::state::SharedState) -> Result<()> {
        tracing::info!("Run command");
        Ok(())
    }
}
