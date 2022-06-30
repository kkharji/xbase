use xbase_proto::*;

mod nvim;
mod runtime;

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

#[macro_export]
macro_rules! request {
    ($method:ident, $($arg:tt)*) => {
        rt().block_on(async move {
            let rpc = rpc().await;
            let ctx = context::current();
            rpc.$method(ctx, $($arg)*).await?
        })
    };
}

#[mlua::lua_module]
fn xbase_client(_lua: &mlua::Lua) -> mlua::Result<impl mlua::UserData> {
    Ok(nvim::XBaseUserData)
}
