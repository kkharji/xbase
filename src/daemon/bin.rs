#[tokio::main]
async fn main() -> anyhow::Result<()> {
    xcodebase::install_tracing("/tmp", "xcodebase-daemon.log", true)?;
    xcodebase::Daemon::default().run().await
}
