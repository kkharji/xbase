use xbase_proto::*;

mod neovim;
mod runtime;

pub trait BrodcastMessage {
    type Result;
    fn parse(&self, slice: &[u8]) -> Result<Message> {
        serde_json::from_slice(slice).map_err(|e| {
            let line = String::from_utf8(slice.to_vec()).unwrap_or_default();
            Error::MessageParse(format!("\n\n\nERROR parsing `{line}`: {e}\n\n\n"))
        })
    }
    fn handle(&self, msg: Message) -> Self::Result;
    fn update_statusline(&self, state: StatuslineState) -> Self::Result;
}

pub trait XBase {
    type Result;

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
    fn register(&self, root: Option<String>) -> Self::Result;
    /// Send build request to client
    fn build(&self, req: BuildRequest) -> Self::Result;
    /// Send run request to client
    fn run(&self, req: RunRequest) -> Self::Result;
    /// Send drop request to client
    fn drop(&self, root: Option<String>) -> Self::Result;
}

#[mlua::lua_module]
fn xbase_editor_lib(lua: &mlua::Lua) -> mlua::Result<neovim::XBaseUserData> {
    Ok(neovim::XBaseUserData)
}
