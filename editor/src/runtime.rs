use once_cell::sync::Lazy;
use std::future::Future;
use std::path::PathBuf;
use std::{net::Shutdown, os::unix::net::UnixStream, process::Command};
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;
use xbase_proto::*;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Tokio runtime"));

static DAEMON_SOCKET_ADDRESS: &str = "/tmp/xbase.socket";

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

static CLIENT: OnceCell<XBaseClient> = OnceCell::const_new();

pub async fn client() -> &'static XBaseClient {
    CLIENT
        .get_or_init(|| async {
            let codec_builder = LengthDelimitedCodec::builder();
            let conn = tokio::net::UnixStream::connect(DAEMON_SOCKET_ADDRESS)
                .await
                .unwrap();
            let transport = transport::new(codec_builder.new_framed(conn), Json::default());
            XBaseClient::new(Default::default(), transport).spawn()
        })
        .await
}

pub async fn block_on<F: Future>(future: F) -> F::Output {
    tokio::task::LocalSet::new()
        .run_until(async move { RUNTIME.block_on(future) })
        .await
}

#[inline]
pub fn ensure_daemon() -> bool {
    if let Ok(stream) = UnixStream::connect(DAEMON_SOCKET_ADDRESS) {
        stream.shutdown(Shutdown::Both).ok();
        false
    } else {
        Command::new(&*DAEMON_BINARY_PATH).spawn().unwrap();
        std::thread::sleep(std::time::Duration::new(1, 0));
        true
    }
}

macro_rules! spawn {
    ($body:block) => {
        $crate::runtime::block_on(async { Ok::<_, Error>($body.await?) })
    };
    ($body:ident) => {
        $crate::runtime::block_on(async { Ok::<_, Error>($body.await?) })
    };
}
pub(crate) use spawn;
