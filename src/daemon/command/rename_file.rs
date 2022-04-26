use anyhow::Result;

/// Rename file + class
#[derive(Debug)]
pub struct RenameFile {
    // TODO: Simplify rename file args by extracting current name from path.
    pub path: String,
    pub name: String,
    pub new_name: String,
}

// TODO: Implement file rename along with it's main class if any.
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl crate::daemon::DaemonCommandExt for RenameFile {
    async fn handle(&self, _state: crate::daemon::DaemonState) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}

impl TryFrom<Vec<&str>> for RenameFile {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
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
}

impl RenameFile {
    pub const KEY: &'static str = "rename_file";
    pub fn request(path: &str, name: &str, new_name: &str) -> Result<()> {
        crate::daemon::Daemon::execute(&[Self::KEY, path, name, new_name])
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
}
