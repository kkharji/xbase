use log::Level;
use std::collections::HashMap;
use std::path::PathBuf;
use tap::Pipe;
use tokio::fs::{metadata, read_to_string, remove_file, write};
use tokio::net::UnixListener;
use xbase::constants::*;
use xbase::util::pid;
use xbase_proto::*;
use xcodeproj::pbxproj::PBXTargetPlatform;

#[derive(Clone)]
struct Server;

#[tarpc::server]
impl xbase_proto::XBase for Server {
    async fn register(self, _: Context, root: PathBuf) -> Result<PathBuf> {
        xbase::register::handle(root).await
    }
    async fn build(self, _: Context, req: BuildRequest) -> Result<()> {
        tokio::spawn(async { xbase::build::handle(req).await });
        Ok(())
    }
    async fn run(self, _: Context, req: RunRequest) -> Result<()> {
        tokio::spawn(async { xbase::run::handle(req).await });
        Ok(())
    }

    async fn watching(self, _: Context, root: PathBuf) -> Result<Vec<String>> {
        let state = DAEMON_STATE.lock().await;
        Ok(state
            .watcher
            .get(&root)?
            .listeners
            .iter()
            .map(|(k, _)| k.clone())
            .collect())
    }

    async fn drop(self, _: Context, root: PathBuf) -> Result<()> {
        tokio::spawn(async { xbase::drop::handle(root).await });
        Ok(())
    }
    async fn targets(self, _: Context, root: PathBuf) -> Result<HashMap<String, TargetInfo>> {
        let state = DAEMON_STATE.lock().await;
        let project = state.projects.get(&root)?;
        Ok(project.targets().clone())
    }
    async fn runners(
        self,
        _: Context,
        platform: PBXTargetPlatform,
    ) -> Result<Vec<HashMap<String, String>>> {
        DAEMON_STATE
            .lock()
            .await
            .devices
            .iter()
            .filter(|(_, d)| d.platform == platform)
            .map(|(id, d)| {
                HashMap::from([("id".into(), id.clone()), ("name".into(), d.name.clone())])
            })
            .collect::<Vec<HashMap<String, String>>>()
            .pipe(Ok)
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
                let channel = BaseChannel::with_defaults(transport);

                channel.execute(Server.serve()).await;
            });
            // TODO: break loop if no more projects
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
    OK(())
}
