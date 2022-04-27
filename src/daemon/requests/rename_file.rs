#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Rename file + class
#[derive(Debug)]
pub struct RenameFile {
    // TODO: Simplify rename file args by extracting current name from path.
    pub path: String,
    pub name: String,
    pub new_name: String,
}

impl RenameFile {
    pub const KEY: &'static str = "rename_file";
}

// TODO: Implement file rename along with it's main class if any.
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<RenameFile> for RenameFile {
    fn parse(args: Vec<&str>) -> Result<Self> {
        let (path, new_name, name) = (args.get(0), args.get(1), args.get(2));
        if path.is_none() || name.is_none() || new_name.is_none() {
            anyhow::bail!(
                "Missing arugments: [path: {:?}, old_name: {:?}, name_name: {:?} ]",
                path,
                name,
                new_name
            )
        }

        Ok(Self {
            path: path.unwrap().to_string(),
            name: name.unwrap().to_string(),
            new_name: new_name.unwrap().to_string(),
        })
    }

    async fn handle(&self, _state: DaemonState) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl RenameFile {
    pub fn lua(
        lua: &mlua::Lua,
        (path, name, new_name): (String, String, String),
    ) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(&format!("Rename command called"))?;
        Self::request(&path, &name, &new_name).map_err(mlua::Error::external)
    }

    pub fn request(path: &str, name: &str, new_name: &str) -> mlua::Result<()> {
        Daemon::execute(&[Self::KEY, path, name, new_name])
    }
}
