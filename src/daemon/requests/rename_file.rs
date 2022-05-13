use super::*;

/// Rename file + class
#[derive(Debug, Serialize, Deserialize)]
pub struct RenameFile {
    pub client: Client,
}

// TODO: Implement file rename along with it's main class if any.
#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for RenameFile {
    async fn handle(self) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, RenameFile> for RenameFile {}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for RenameFile {
    fn from_lua(v: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = v {
            Ok(Self {
                client: table.get("client")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize RenameFile"))
        }
    }
}
