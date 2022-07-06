use super::request;
mod broadcast;
mod global;

pub use broadcast::*;
pub use global::*;

use crate::{runtime::*, BroadcastHandler};
use mlua::{chunk, prelude::*};
use std::{collections::HashSet, path::PathBuf, str::FromStr};
use tap::Pipe;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

pub struct XBaseUserData;

impl mlua::UserData for XBaseUserData {
    fn add_methods<'lua, M>(m: &mut M)
    where
        M: mlua::UserDataMethods<'lua, Self>,
    {
        m.add_function("build", |_lua, req: BuildRequest| {
            request!(build, req).to_lua_err()
        });

        m.add_function("run", |_lua, req: RunRequest| {
            request!(run, req).to_lua_err()
        });

        m.add_function("drop", |_lua, root: Option<String>| {
            let mut curr_roots = roots().lock().unwrap();
            let roots = if let Some(root) = root {
                let root = PathBuf::from(root);
                curr_roots.remove(&root);
                HashSet::from([root])
            } else {
                let roots = HashSet::from_iter(curr_roots.iter().map(Clone::clone));
                curr_roots.clear();
                roots
            };
            request!(drop, roots).to_lua_err()
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
