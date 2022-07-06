use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{net::Shutdown, os::unix::net::UnixStream, process::Command};
use tokio::{runtime::Runtime, sync::OnceCell};
use xbase_proto::*;

/// ROOTS with their reader file descriptor and thread handling the writing
static ROOTS: Lazy<Mutex<HashMap<PathBuf, (i32, JoinHandle<Result<()>>)>>> =
    Lazy::new(Default::default);
static CLIENT: OnceCell<XBaseClient> = OnceCell::const_new();
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

/// Get Tokio Runtime
pub fn rt() -> &'static Runtime {
    &*RUNTIME
}

/// Get Registered roots
pub fn roots() -> &'static Mutex<HashMap<PathBuf, (i32, JoinHandle<Result<()>>)>> {
    &*ROOTS
}

/// Get RPC to make request to xbase daemon
pub async fn rpc() -> &'static XBaseClient {
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

#[macro_export]
macro_rules! request {
    ($method:ident, $($arg:tt)*) => {
        crate::runtime::rt().block_on(async move {
            let rpc = crate::runtime::rpc().await;
            let ctx = xbase_proto::context::current();
            rpc.$method(ctx, $($arg)*).await
        })
    };
}
pub(crate) use request;

/// Generate headers
#[::safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> ::std::io::Result<()> {
    let out_path = "../build/libxbase.h";
    safer_ffi::headers::builder()
        .to_file(out_path)?
        .generate()?;
    let gcc_output = std::process::Command::new("gcc")
        .args(&["-xc", "-E", "-P", out_path])
        .output()?;
    if !gcc_output.status.success() {
        panic!("Failed to gcc process ../build/libxbase.h");
    }
    std::fs::write(out_path, gcc_output.stdout)?;
    Ok(())
}
