mod extensions;
mod logger;
mod state;

use crate::runtime::*;
use mlua::prelude::*;
use xbase_proto::*;

pub(self) use extensions::*;
pub(self) use logger::*;
pub(self) use state::*;

pub struct NeovimDaemonClient;

impl NeovimDaemonClient {
    /// Register current runtime as a daemon client
    ///
    /// If root is given then the registration will be for the root instead
    pub async fn register(lua: &Lua, root: Option<String>) -> Result<()> {
        let client = client().await;
        if ensure_daemon() {
            lua.info("new instance initialized, connecting ..")?;
        }

        let req = RegisterRequest {
            client: Client::new(lua, root)?,
        };

        let _log_path = client.register(context::current(), req).await??;

        lua.info("Connected")?;

        Ok(())
    }

    /// Build project
    pub async fn build(_lua: &Lua, req: BuildRequest) -> Result<()> {
        let client = client().await;
        let ctx = context::current();
        let _path = client.build(ctx, req).await??;
        Ok(())
    }

    /// Run project
    pub async fn run(_lua: &Lua, req: RunRequest) -> Result<()> {
        let client = client().await;
        let ctx = context::current();
        let _path = client.run(ctx, req).await??;
        Ok(())
    }

    /// Drop project at given root, otherwise drop client
    pub async fn drop(lua: &Lua, root: Option<String>) -> Result<()> {
        let client = client().await;
        let ctx = context::current();
        let req = DropRequest {
            remove_client: root.is_none(),
            client: Client::new(lua, root)?,
        };

        client.drop(ctx, req).await??;
        Ok(())
    }
}
