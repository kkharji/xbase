use std::collections::HashMap;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

mod neovim;
mod runtime;

pub trait Broadcast {
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

pub trait XBase {
    type Error;

    /// Register project on the client.
    ///
    /// This function start a listener for daemon commands to either log messages or excitation
    /// actions for every new project root. If root already registered, then
    ///
    /// Note [`Listener::init_or_skip`] will skip creating a listener when root is already
    /// registered.
    ///
    /// See [`Listener::start_reader`] to understand how the client handle messages.
    ///
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

#[mlua::lua_module]
fn xbase_editor_lib(_lua: &mlua::Lua) -> mlua::Result<neovim::XBaseUserData> {
    Ok(neovim::XBaseUserData)
}
