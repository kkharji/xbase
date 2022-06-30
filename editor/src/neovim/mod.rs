mod global;
mod listener;
mod state;
use crate::{runtime::*, Broadcast, XBase};
use mlua::{chunk, prelude::*};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

pub use global::*;
pub use listener::*;
pub use state::*;

pub struct XBaseUserData;

impl LuaUserData for XBaseUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(m: &mut M) {
        m.add_function("register", XBase::register);
        m.add_function("build", XBase::build);
        m.add_function("run", XBase::run);
        m.add_function("drop", XBase::drop);
        m.add_function("targets", |lua, root: Option<String>| {
            lua.create_table_from(lua.targets(root)?)
        });
        m.add_function("runners", |lua, platform: String| {
            let runners = lua.runners(PBXTargetPlatform::from_str(&platform).to_lua_err()?)?;
            let table = lua.create_table()?;
            for (i, runner) in runners.into_iter().enumerate() {
                table.set(i, lua.create_table_from(runner)?)?;
            }
            Ok(table)
        });
        m.add_function("log_bufnr", |lua, _: ()| lua.log_bufnr());
        m.add_function("watching", |lua, root: Option<String>| {
            lua.create_table_from(lua.watching(root)?.into_iter().enumerate())
        });
    }
}

impl Broadcast for Lua {
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

    fn update_statusline(&self, _state: StatuslineState) -> Self::Result {
        todo!()
    }
}

impl XBase for Lua {
    type Error = LuaError;

    fn register(&self, root: Option<String>) -> LuaResult<()> {
        if ensure_daemon() {
            self.info("new instance initialized")?;
        }

        let root = self.root(root)?;

        Listener::init_or_skip(self, &root)?;

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

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }

    fn api(&self) -> LuaResult<LuaTable> {
        self.vim()?.get("api")
    }

    fn root(&self, root: Option<String>) -> LuaResult<PathBuf> {
        let root = match root {
            Some(root) => root,
            None => self
                .vim()?
                .get::<_, LuaTable>("loop")?
                .get::<_, LuaFunction>("cwd")?
                .call::<_, String>(())?,
        };
        Ok(PathBuf::from(root))
    }

    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()> {
        if msg.as_ref().trim().is_empty() {
            return Ok(());
        }
        let notify: LuaFunction = self.vim()?.get("notify")?;
        let msg = format!("xbase: {}", msg.as_ref());
        notify.call((msg, level as u8))
    }

    // TODO: Respect user configuration, Only log the level the user set.
    // TODO: Change line color based on level
    // TODO: Fix first line is empty
    fn log<S: AsRef<str>>(&self, msg: S, _level: MessageLevel) -> LuaResult<()> {
        let msg = msg.as_ref().trim();

        if msg.is_empty() {
            return Ok(());
        }
        let bufnr = self.state()?.bufnr;
        let api = self.api()?;

        let set_lines: LuaFunction = api.get("nvim_buf_set_lines")?;
        let is_empty: bool = self
            .load(chunk!( return vim.api.nvim_buf_line_count($bufnr) == 1 and vim.api.nvim_buf_get_lines($bufnr, 0, 1, false)[1] == ""))
            .eval()?;

        set_lines.call((bufnr, if is_empty { 0 } else { -1 }, -1, false, vec![msg]))?;

        Ok(())
    }
}
