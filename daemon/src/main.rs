use log::Level;
use tap::Pipe;
use tokio::fs::{metadata, read_to_string, remove_file, write};
use tokio::net::UnixListener;
use xbase::constants::*;
use xbase::util::pid;
use xbase_proto::*;

#[derive(Clone)]
struct Server;

#[tarpc::server]
impl xbase_proto::XBase for Server {
    async fn register(self, _: Context, req: RegisterRequest) -> Result<Vec<LoggingTask>> {
        xbase::register::handle(req).await
    }

    async fn build(self, _: Context, req: BuildRequest) -> Result<LoggingTask> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { xbase::build::handle(req).await });
        Ok(LoggingTask::default())
    }

    async fn run(self, _: Context, req: RunRequest) -> Result<LoggingTask> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { xbase::run::handle(req).await });
        Ok(Default::default())
    }

    async fn drop(self, _: Context, req: DropRequest) -> Result<()> {
        // NOTE: Required because of nvim-rs
        tokio::spawn(async { xbase::drop::handle(req).await });
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
