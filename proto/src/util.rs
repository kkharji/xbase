use serde::{Deserialize, Deserializer};

/// Deserialize an optional type to default if none
pub fn value_or_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

macro_rules! into_request {
    ($type:ident) => {
        paste::paste! {
            impl From<[<$type Request>]> for crate::Request {
                fn from(msg: [<$type Request>]) -> Self {
                    let message = crate::Message::$type(msg);
                    Self { message }
                }
            }
        }
    };
}
pub(crate) use into_request;
#[cfg(feature = "neovim")]
use mlua::prelude::*;

#[cfg(feature = "neovim")]
pub(crate) fn cwd(lua: &Lua) -> LuaResult<String> {
    lua.globals()
        .get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("loop")?
        .get::<_, LuaFunction>("cwd")?
        .call::<_, String>(())
}

#[cfg(feature = "neovim")]
pub(crate) fn address(lua: &Lua) -> LuaResult<String> {
    let global = lua.globals();
    let address = match global.get::<_, LuaString>("__SERVERNAME") {
        Ok(v) => v,
        Err(_) => {
            let value = global
                .get::<_, LuaTable>("vim")
                .and_then(|v| v.get::<_, LuaTable>("v"))
                .and_then(|v| v.get::<_, LuaString>("servername"))?;
            global.set("__SERVERNAME", value.clone())?;
            value
        }
    }
    .to_string_lossy()
    .to_string();
    Ok(address)
}
