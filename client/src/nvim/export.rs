use super::{Broadcast, NvimGlobal, XBaseStateExt};
use crate::{runtime::*, XBase};
use mlua::prelude::*;
use std::{collections::HashMap, str::FromStr};
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

impl XBase for Lua {
    type Error = LuaError;

    fn register(&self, root: Option<String>) -> LuaResult<()> {
        let root = self.root(root)?;
        if !(root.join("project.yml").exists()
            || root.join("Project.swift").exists()
            || root.join("Package.swift").exists()
            || wax::walk("*.xcodeproj", &root).to_lua_err()?.count() != 0)
        {
            return Ok(());
        }
        if ensure_daemon() {
            self.info("new instance initialized")?;
        }

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

pub struct XBaseUserData;

impl mlua::UserData for XBaseUserData {
    fn add_methods<'lua, M>(m: &mut M)
    where
        M: mlua::UserDataMethods<'lua, Self>,
    {
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
