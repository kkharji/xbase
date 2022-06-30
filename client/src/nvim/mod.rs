mod broadcast;
mod global;
mod state;
use crate::{runtime::*, BroadcastHandler, XBase};
use mlua::{chunk, prelude::*};
use std::collections::HashMap;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

pub use broadcast::*;
pub use global::*;
pub use state::*;

impl BroadcastHandler for Lua {
    type Result = LuaResult<()>;

    fn handle(&self, msg: Message) -> Self::Result {
        match msg {
            Message::Notify { msg, level, .. } => self.notify(msg, level),
            Message::Log { msg, level, .. } => self.log(msg, level),
            Message::Execute(task) => match task {
                Task::UpdateStatusline(value) => self.update_statusline(value),
            },
        }
    }

    fn update_statusline(&self, state: StatuslineState) -> Self::Result {
        let value = state.to_string();
        self.load(chunk!(vim.g.xbase_watch_build_status = $value))
            .exec()
    }
}

impl XBase for Lua {
    type Error = LuaError;

    fn register(&self, root: Option<String>) -> LuaResult<()> {
        if ensure_daemon() {
            self.info("new instance initialized")?;
        }

        let root = self.root(root)?;

        Broadcast::init_or_skip(self, &root)?;

        // Setup State (skipped if already set)
        self.setup_state()?;

        self.info(format!("[{}] Connected ", root.as_path().name().unwrap()))?;

        Ok(())
    }

    /// Build project
    fn build(&self, req: BuildRequest) -> LuaResult<()> {
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            let _path = rpc.build(ctx, req).await??;
            OK(())
        })
        .to_lua_err()?;
        Ok(())
    }

    /// Run project
    fn run(&self, req: RunRequest) -> LuaResult<()> {
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            let _path = rpc.run(ctx, req).await??;
            OK(())
        })
        .to_lua_err()?;
        Ok(())
    }

    /// Drop project at given root, otherwise drop client
    fn drop(&self, root: Option<String>) -> LuaResult<()> {
        let root = self.root(root)?;
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();

            rpc.drop(ctx, root).await??;

            OK(())
        })?;
        Ok(())
    }

    /// Get targets for given root
    fn targets(&self, root: Option<String>) -> LuaResult<HashMap<String, TargetInfo>> {
        let root = self.root(root)?;
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            rpc.targets(ctx, root).await?
        })
        .to_lua_err()
    }

    /// Get targets for given root
    fn runners(&self, platform: PBXTargetPlatform) -> LuaResult<Vec<HashMap<String, String>>> {
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            rpc.runners(ctx, platform).await?
        })
        .to_lua_err()
    }

    /// Get a vector of keys being watched
    fn watching(&self, root: Option<String>) -> LuaResult<Vec<String>> {
        let root = self.root(root)?;
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            rpc.watching(ctx, root).await?
        })
        .to_lua_err()
    }
}
