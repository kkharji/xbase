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

/// Broadcast server function handler.
///
/// See [`nvim::Broadcast`]
pub trait BroadcastHandler {
    type Result;
    fn handle(&self, msg: Message) -> Self::Result;

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
}

#[mlua::lua_module]
fn xbase_client(_lua: &mlua::Lua) -> mlua::Result<impl mlua::UserData> {
    Ok(nvim::XBaseUserData)
}
