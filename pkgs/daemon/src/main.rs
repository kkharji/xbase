#[tokio::main]
async fn main() -> anyhow::Result<()> {
    xcodebase::util::install_tracing("/tmp", "xcodebase-daemon.log")?;
    xcodebase::Daemon::default().run().await
}
