use super::request;
mod broadcast;
mod global;

pub use broadcast::*;
pub use global::*;

use crate::{runtime::*, BroadcastHandler};
use mlua::{chunk, prelude::*};
use std::str::FromStr;
use tap::Pipe;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

pub struct XBaseUserData;

impl BroadcastHandler for Lua {
    type Result = LuaResult<()>;

    fn handle(&self, msg: Message) -> Self::Result {
        match msg {
            Message::Notify { msg, level, .. } => self.notify(msg, level),
            Message::Log { msg, level, .. } => self.log(msg, level),
            Message::Execute(task) => match task {
                Task::UpdateStatusline(state) => state.to_string().pipe(|s| {
                    self.load(chunk!(vim.g.xbase_watch_build_status = $s))
                        .exec()
                }),
                Task::OpenLogger => {
                    // TODO: Make auto open logger on error configurable
                    self.load(chunk!(require("xbase.log").toggle())).exec()
                }
            },
        }
    }
}

impl mlua::UserData for XBaseUserData {
    fn add_methods<'lua, M>(m: &mut M)
    where
        M: mlua::UserDataMethods<'lua, Self>,
    {
        m.add_function("register", |lua, root: Option<String>| -> LuaResult<bool> {
            let root = lua.root(root)?;

            if !(root.join("project.yml").exists()
                || root.join("Project.swift").exists()
                || root.join("Package.swift").exists()
                || wax::walk("*.xcodeproj", &root).to_lua_err()?.count() != 0)
            {
                return Ok(false);
            }

            if ensure_daemon() {
                lua.info("new instance initialized")?;
            }

            Broadcast::init_or_skip(lua, &root)?;

            lua.load(chunk!(require("xbase.log").setup())).exec()?;

            lua.info(format!("[{}] Connected ", root.as_path().name().unwrap()))?;

            Ok(true)
        });

        m.add_function("build", |_lua, req: BuildRequest| {
            request!(build, req).to_lua_err()
        });

        m.add_function("run", |_lua, req: RunRequest| {
            request!(run, req).to_lua_err()
        });

        m.add_function("drop", |lua, root: Option<String>| {
            request!(drop, lua.root(root)?).to_lua_err()
        });

        m.add_function("targets", |lua, root: Option<String>| {
            let targets = request!(targets, lua.root(root)?).to_lua_err()?;
            lua.create_table_from(targets)
        });

        m.add_function("runners", |lua, platform: String| {
            let platform = PBXTargetPlatform::from_str(&platform).to_lua_err()?;
            let runners = request!(runners, platform).to_lua_err()?;

            let table = lua.create_table()?;
            for (i, runner) in runners.into_iter().enumerate() {
                table.set(i, lua.create_table_from(runner)?)?;
            }
            Ok(table)
        });

        m.add_function("watching", |lua, root: Option<String>| {
            let watching = request!(watching, lua.root(root)?).to_lua_err()?;
            lua.create_table_from(watching.into_iter().enumerate())
        });
    }
}
