use std::collections::HashMap;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

mod nvim;
mod runtime;
pub trait XBase {
    type Error;
    /// Register project at given root or cwd.
    fn register(&self, root: Option<String>) -> Result<(), Self::Error>;
    /// Send build request to client
    fn build(&self, req: BuildRequest) -> Result<(), Self::Error>;
    /// Send run request to client
    fn run(&self, req: RunRequest) -> Result<(), Self::Error>;
    /// Send drop request to client
    fn drop(&self, root: Option<String>) -> Result<(), Self::Error>;
    /// Send targets for given root
    fn targets(&self, root: Option<String>) -> Result<HashMap<String, TargetInfo>, Self::Error>;
    /// Send targets for given root
    fn runners(
        &self,
        platform: PBXTargetPlatform,
    ) -> Result<Vec<HashMap<String, String>>, Self::Error>;
    /// Get currently watched tagets and configuration
    fn watching(&self, root: Option<String>) -> Result<Vec<String>, Self::Error>;
}

pub trait BroadcastHandler {
    type Result;
    fn parse(&self, content: String) -> Result<Vec<Message>> {
        fn parse_single(line: &str) -> Result<Message> {
            serde_json::from_str(&line)
                .map_err(|e| Error::MessageParse(format!("\n{line}\n Error: {e}")))
        }
        let mut vec = vec![];
        if content.contains("\n") {
            for message in content
                .split("\n")
                .filter(|s| !s.trim().is_empty())
                .map(parse_single)
            {
                vec.push(message?);
            }
        } else {
            vec.push(parse_single(&content)?);
        }
        Ok(vec)
    }
    fn handle(&self, msg: Message) -> Self::Result;
    fn update_statusline(&self, state: StatuslineState) -> Self::Result;
}

#[mlua::lua_module]
fn xbase_client(_lua: &mlua::Lua) -> mlua::Result<impl mlua::UserData> {
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
                let runners = lua.runners(mlua::ExternalResult::to_lua_err(
                    <PBXTargetPlatform as std::str::FromStr>::from_str(&platform),
                )?)?;
                let table = lua.create_table()?;
                for (i, runner) in runners.into_iter().enumerate() {
                    table.set(i, lua.create_table_from(runner)?)?;
                }
                Ok(table)
            });
            m.add_function("log_bufnr", |lua, _: ()| {
                nvim::XBaseStateExt::log_bufnr(lua)
            });
            m.add_function("watching", |lua, root: Option<String>| {
                lua.create_table_from(lua.watching(root)?.into_iter().enumerate())
            });
        }
    }
    Ok(XBaseUserData)
}
