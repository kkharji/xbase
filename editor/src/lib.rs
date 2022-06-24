mod extension;
use extension::*;
use mlua::{lua_module, prelude::*};
use once_cell::sync::Lazy;
use std::future::Future;
use std::path::PathBuf;
use std::{net::Shutdown, os::unix::net::UnixStream, process::Command};
use tokio::runtime::Runtime;
use xbase_proto::*;

static DAEMON_SOCKET_PATH: &str = "/tmp/xbase.socket";

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    // Must Create!
    Runtime::new().expect("Should create a tokio runtime")
});

static CLIENT: Lazy<XBaseClient> = Lazy::new(|| {
    RUNTIME.block_on(async {
        let codec_builder = LengthDelimitedCodec::builder();
        let conn = tokio::net::UnixStream::connect(DAEMON_SOCKET_PATH)
            .await
            .unwrap();
        let transport = transport::new(codec_builder.new_framed(conn), Json::default());
        XBaseClient::new(Default::default(), transport).spawn()
    })
});

static DAEMON_BINARY_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    if cfg!(debug_assertions) {
        root.extend(&["target", "debug", "xbase"]);
    } else {
        root.extend(&["bin", "xbase"]);
    }
    root
});

async fn block_on<F: Future>(future: F) -> F::Output {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move { RUNTIME.block_on(future) })
        .await
}

fn client() -> &'static XBaseClient {
    &*CLIENT
}

macro_rules! spawn {
    ($body:block) => {
        block_on(async { Ok::<_, Error>($body.await?) })
    };
}

struct DaemonClient;

impl LuaUserData for DaemonClient {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ensure", |lua, _: ()| {
            if let Ok(stream) = UnixStream::connect(DAEMON_SOCKET_PATH) {
                stream.shutdown(Shutdown::Both).ok();
            } else {
                Command::new(&*DAEMON_BINARY_PATH).spawn().unwrap();
                std::thread::sleep(std::time::Duration::new(1, 0));
                lua.info(&"XBase: daemon initialized");
            }
            Ok(())
        });

        methods.add_async_function("register", |lua, root: Option<String>| async move {
            let client = client();
            let req = RegisterRequest {
                client: Client::new(lua, root)?,
            };

            let _path = spawn!({ client.register(context::current(), req) }).await??;

            lua.info(&"XBase: ✅");

            Ok(())
        });

        methods.add_async_function("build", |_, req: BuildRequest| async move {
            let client = client();
            let ctx = context::current();
            let _path = spawn!({ client.build(ctx, req) }).await??;

            Ok(())
        });

        methods.add_async_function("run", |_, req: RunRequest| async move {
            let client = client();
            let ctx = context::current();
            let _path = spawn!({ client.run(ctx, req) }).await??;

            Ok(())
        });

        methods.add_async_function("drop", |lua, root: Option<String>| async move {
            let client = client();
            let ctx = context::current();
            let req = DropRequest {
                remove_client: root.is_none(),
                client: Client::new(lua, root)?,
            };

            spawn!({ client.drop(ctx, req) }).await??;

            Ok(())
        });
    }
}

#[lua_module]
fn xbase_editor_lib(_: &Lua) -> LuaResult<DaemonClient> {
    Ok(DaemonClient)
}
