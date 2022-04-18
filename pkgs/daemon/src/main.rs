#[tokio::main]
async fn main() -> anyhow::Result<()> {
    xcodebase::install_tracing("/tmp", "xcodebase-daemon.log")?;
    xcodebase::Daemon::default().run().await
}
