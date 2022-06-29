mod global;
mod listener;
mod state;

use std::path::PathBuf;

use crate::{runtime::*, Broadcast, XBase};
use mlua::{chunk, prelude::*};
use xbase_proto::*;

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
        m.add_function("log_bufnr", |lua, _: ()| Ok(lua.state()?.bufnr));
    }
}

impl Broadcast for Lua {
    type Result = LuaResult<()>;

    fn handle(&self, msg: Message) -> Self::Result {
        if msg.should_skip(std::process::id()) {
            return Ok(());
        };

        match msg {
            Message::Notify { msg, level, .. } => self.notify(msg, level),
            Message::Log { msg, level, .. } => self.log(msg, level),
            Message::Execute { task, .. } => match task {
                Task::UpdateStatusline(value) => self.update_statusline(value),
            },
        }
    }

    fn update_statusline(&self, _state: StatuslineState) -> Self::Result {
        todo!()
    }
}

impl XBase for Lua {
    type Result = LuaResult<()>;

    fn register(&self, root: Option<String>) -> Self::Result {
        if ensure_daemon() {
            self.info("new instance initialized, connecting ..")?;
        }

        let root = if let Some(root) = root {
            root.into()
        } else {
            self.cwd()?
        };

        Listener::init_or_skip(self, root)?;

        // Setup State (skipped if already set)
        self.setup_state()?;

        Ok(())
    }

    /// Build project
    fn build(&self, req: BuildRequest) -> Self::Result {
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
    fn run(&self, req: RunRequest) -> Self::Result {
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
    fn drop(&self, root: Option<String>) -> Self::Result {
        let client = Client::new(self, root)?;
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();

            rpc.drop(ctx, client).await??;

            OK(())
        })?;
        Ok(())
    }
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }

    fn api(&self) -> LuaResult<LuaTable> {
        self.vim()?.get("api")
    }

    fn cwd(&self) -> LuaResult<PathBuf> {
        self.vim()?
            .get::<_, LuaTable>("loop")?
            .get::<_, LuaFunction>("cwd")?
            .call::<_, String>(())
            .map(PathBuf::from)
    }

    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()> {
        let notify: LuaFunction = self.vim()?.get("notify")?;
        notify.call((msg.as_ref(), level as u8))
    }

    // TODO: Respect user configuration, Only log the level the user set.
    // TODO: Change line color based on level
    // TODO: Fix first line is empty
    fn log<S: AsRef<str>>(&self, msg: S, _level: MessageLevel) -> LuaResult<()> {
        let msg = msg.as_ref().to_string();
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
