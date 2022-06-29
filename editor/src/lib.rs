use xbase_proto::*;

mod neovim;
mod runtime;

pub trait BrodcastMessage {
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
