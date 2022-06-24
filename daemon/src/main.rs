use log::Level;
use std::path::PathBuf;
use tap::Pipe;
use tokio::fs::{metadata, read_to_string, remove_file, write};
use tokio::net::UnixListener;
use xbase::util::pid;
use xbase::{constants::*, RequestHandler};
use xbase_proto::*;

#[derive(Clone)]
struct Server;

#[tarpc::server]
impl xbase_proto::XBase for Server {
    /// Register project root with a path to setup logs
    async fn register(self, _: Context, req: RegisterRequest) -> Result<PathBuf> {
        req.handle().await?;
        Ok("/bin/cp".into())
    }
    /// Build Project and get path to where to build log will be located
    async fn build(self, _: Context, req: BuildRequest) -> Result<PathBuf> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { req.handle().await });
        Ok(PathBuf::default())
    }
    /// Run Project and get path to where to Runtime log will be located
    async fn run(self, _: Context, req: RunRequest) -> Result<PathBuf> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { req.handle().await });
        Ok(PathBuf::default())
    }
    /// Drop project root
    async fn drop(self, _: Context, req: DropRequest) -> Result<()> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { req.handle().await });
        Ok(())
    }
}
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    ensure_single_instance().await?;

    let listener = UnixListener::bind(DAEMON_SOCKET_PATH).unwrap();
    let codec_builder = LengthDelimitedCodec::builder();
    log::setup("/tmp", "xbase-daemon.log", Level::DEBUG, true)?;
    log::info!("Started");

    loop {
        if let Ok((s, _)) = listener.accept().await {
            tokio::spawn(async move {
                let framed = codec_builder.new_framed(s);
                let transport = transport::new(framed, Json::default());
                BaseChannel::with_defaults(transport)
                    .execute(Server.serve())
                    .await;

                let state = DAEMON_STATE.clone();
                let mut state = state.lock().await;
                state.validate().await;
            });
        } else {
            log::error!("Fail to accept a connection")
        };
    }
}

async fn ensure_single_instance() -> Result<()> {
    if metadata(DAEMON_SOCKET_PATH).await.ok().is_some() {
        remove_file(DAEMON_SOCKET_PATH).await.ok();
        if metadata(DAEMON_PID_PATH).await.ok().is_some() {
            read_to_string(DAEMON_PID_PATH)
                .await?
                .pipe_ref(pid::kill)
                .await?;
        }
        remove_file(DAEMON_PID_PATH).await.ok();
    }
    write(DAEMON_PID_PATH, std::process::id().to_string()).await?;
    Ok(())
}
