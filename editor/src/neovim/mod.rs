mod global;
mod listener;
mod state;

use std::path::PathBuf;

use crate::{runtime::*, BrodcastMessage, XBase};
use mlua::prelude::*;
use xbase_proto::*;

pub use global::*;
pub use listener::*;
pub use state::*;

impl BrodcastMessage for Lua {
    type Result = LuaResult<()>;

    fn handle(&self, msg: Message) -> Self::Result {
        match msg {
            Message::Notify { msg, level } => self.notify(msg, level),
            Message::Log { msg, level } => self.notify(msg, level),
            Message::Execute(task) => match task {
                Task::UpdateStatusline(state) => self.update_statusline(state),
            },
        }
    }

    fn update_statusline(&self, state: StatuslineState) -> Self::Result {
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

pub struct XBaseUserData;

impl LuaUserData for XBaseUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(m: &mut M) {
        m.add_function("register", XBase::register);
        m.add_function("build", XBase::build);
        m.add_function("run", XBase::run);
        m.add_function("drop", XBase::drop);
    }
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
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

    fn log<S: AsRef<str>>(&self, _: S, _: MessageLevel) -> std::result::Result<(), mlua::Error> {
        todo!()
    }
}
