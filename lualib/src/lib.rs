mod extension;
use extension::*;

use mlua::{lua_module, prelude::*};
use std::path::PathBuf;
use std::{net::Shutdown, os::unix::net::UnixStream, process::Command, str::FromStr};
use tap::Pipe;
use xbase_proto::*;

static DAEMON_SOCKET_PATH: &str = "/tmp/xbase-daemon.socket";

lazy_static::lazy_static! {
    static ref DAEMON_BINARY_PATH: PathBuf = {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
        if cfg!(debug_assertions) {
            root.extend(&["target", "debug", "xbase-daemon"]);
        } else {
            root.extend(&["bin", "xbase-daemon"]);
        }
        root
    };
}

/// Check if Daemon is running
fn is_running(_: &Lua, _: ()) -> LuaResult<bool> {
    Ok(match UnixStream::connect(DAEMON_SOCKET_PATH) {
        Ok(stream) => stream.shutdown(Shutdown::Both).ok().is_some(),
        Err(_) => false,
    })
}

/// Ensure that daemon is currently running in background
fn ensure(lua: &Lua, _: ()) -> LuaResult<bool> {
    if is_running(lua, ()).unwrap() {
        Ok(false)
    } else {
        Command::new(&*DAEMON_BINARY_PATH).spawn().unwrap();
        // Give time for the daemon to be started
        std::thread::sleep(std::time::Duration::new(1, 0));
        lua.info("Spawned Background Server")?;
        Ok(true)
    }
}

fn request(msg: impl Into<Request>) -> LuaResult<()> {
    use std::io::Write;

    let req: Request = msg.into();
    let mut stream = UnixStream::connect(DAEMON_SOCKET_PATH)
        .map_err(|e| format!("Connect: {e} and execute: {:#?}", req))
        .to_lua_err()?;

    serde_json::to_vec(&req)
        .map(|value| stream.write_all(&value))
        .to_lua_err()??;

    stream.flush().to_lua_err()
}

#[lua_module]
fn libxbase(lua: &Lua) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("is_running", lua.create_function(is_running)?)?;
    table.set("ensure", lua.create_function(ensure)?)?;
    table.set("register", {
        lua.create_function(|lua: &Lua, value: Option<LuaValue>| {
            let client = get_client(lua, value)?;
            request(RegisterRequest { client })
        })
    }?)?;
    table.set("drop", {
        lua.create_function(|lua: &Lua, table: LuaTable| {
            request(DropRequest {
                client: get_client(lua, None)?,
                remove_client: table.get("remove_client")?,
            })
        })
    }?)?;
    table.set("build", {
        lua.create_function(|lua: &Lua, table: LuaTable| {
            request(BuildRequest {
                client: get_client(lua, None)?,
                settings: get_settings(&table)?,
                direction: get_direction(&table)?,
                ops: get_operation(&table)?,
            })
        })
    }?)?;
    table.set("run", {
        lua.create_function(|lua: &Lua, table: LuaTable| {
            request(RunRequest {
                client: get_client(lua, None)?,
                settings: get_settings(&table)?,
                direction: get_direction(&table)?,
                device: get_device(&table)?,
                ops: get_operation(&table)?,
            })
        })
    }?)?;

    Ok(table)
}

/// Create Client from optional value
fn get_client(lua: &Lua, value: Option<LuaValue>) -> LuaResult<Client> {
    let root = match value {
        Some(LuaValue::Table(ref table)) => table.get("root")?,
        Some(LuaValue::String(ref root)) => root.to_string_lossy().to_string(),
        _ => lua.cwd()?,
    };

    Ok(Client {
        pid: std::process::id() as i32,
        address: lua.nvim_address()?,
        root: root.into(),
    })
}

/// Extract BuildSettings from LuaTable
fn get_settings(table: &LuaTable) -> LuaResult<BuildSettings> {
    let table: LuaTable = table.get("settings")?;

    Ok(BuildSettings {
        target: table.get("target")?,
        configuration: table
            .get::<_, String>("configuration")?
            .pipe(|s| BuildConfiguration::from_str(&s))
            .to_lua_err()?,
        scheme: table.get("scheme")?,
    })
}

/// Extract BufferDirection from LuaTable
fn get_direction(table: &LuaTable) -> LuaResult<BufferDirection> {
    let value: LuaValue = table.get("direction")?;

    match value {
        LuaValue::String(value) => value,
        _ => return Err(LuaError::external("Fail to deserialize BufferDirection")),
    }
    .to_string_lossy()
    .pipe(|s| BufferDirection::from_str(&s))
    .to_lua_err()
}

/// Extract Operation from LuaTable
fn get_operation(table: &LuaTable) -> LuaResult<Operation> {
    let value: LuaValue = table.get("ops")?;
    if let LuaValue::String(value) = value {
        let value = value.to_string_lossy();
        Operation::from_str(&*value).to_lua_err()
    } else {
        Ok(Operation::default())
    }
}

/// Extract DeviceLookup from LuaTable
fn get_device(table: &LuaTable) -> LuaResult<DeviceLookup> {
    let value: Option<LuaTable> = table.get("device")?;
    value
        .map(|d| {
            let name = d.get("name").ok()?;
            let udid = d.get("udid").ok()?;
            Some(DeviceLookup { name, udid })
        })
        .flatten()
        .unwrap_or_default()
        .pipe(Ok)
}
