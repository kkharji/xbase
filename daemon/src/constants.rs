use once_cell::sync::Lazy;
use std::path::PathBuf;

/// Where the daemon socket path will be located
pub static DAEMON_SOCKET_PATH: &str = "/tmp/xbase.socket";

/// Where the daemon pid will be located
pub static DAEMON_PID_PATH: &str = "/tmp/xbase.pid";

/// Where the server binary will be located.
pub static BUILD_SERVER_CONFIG: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    if cfg!(debug_assertions) {
        root.extend(&["target", "debug", "xbase-sourcekit-helper"]);
    } else {
        root.extend(&["bin", "xbase-sourcekit-helper"]);
    }
    let json = serde_json::json!({
        "name": "Xbase",
        "argv": [root],
        "version": "0.2",
        "bspVersion": "0.2",
        "languages": ["swift", "objective-c", "objective-cpp", "c", "cpp"]
    });
    json.to_string().into_bytes()
});
