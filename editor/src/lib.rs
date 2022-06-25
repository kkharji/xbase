mod neovim;
mod runtime;

#[mlua::lua_module]
fn xbase_editor_lib(_: &mlua::Lua) -> mlua::Result<neovim::NeovimDaemonClient> {
    use mlua::prelude::*;
    use neovim::NeovimDaemonClient as C;
    use xbase_proto::*;

    impl LuaUserData for C {
        fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(m: &mut M) {
            m.add_async_function("register", |lua, root: Option<String>| async move {
                let register = C::register(lua, root);
                runtime::spawn!(register).await.to_lua_err()
            });

            m.add_async_function("build", |lua, req: BuildRequest| async move {
                let build = C::build(lua, req);
                runtime::spawn!(build).await.to_lua_err()
            });

            m.add_async_function("run", |lua, req: RunRequest| async move {
                let run = C::run(lua, req);
                runtime::spawn!(run).await.to_lua_err()
            });

            m.add_async_function("drop", |lua, root: Option<String>| async move {
                let drop = C::drop(lua, root);
                runtime::spawn!(drop).await.to_lua_err()
            });
        }
    }
    Ok(C)
}
