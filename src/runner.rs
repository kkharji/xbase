mod simctl;

pub use self::simctl::*;

use crate::{
    constants::DAEMON_STATE,
    nvim::BufferDirection,
    state::State,
    types::{Client, Platform},
    util::fmt,
    Error, Result,
};
use {
    process_stream::{Process, ProcessItem, StreamExt},
    tap::Pipe,
    tokio::{sync::OwnedMutexGuard, task::JoinHandle},
    xcodebuild::parser::BuildSettings,
};

pub struct Runner {
    pub client: Client,
    pub target: String,
    pub platform: Platform,
    pub state: OwnedMutexGuard<State>,
    pub udid: Option<String>,
    pub direction: Option<BufferDirection>,
    pub args: Vec<String>,
}

impl Runner {
    pub async fn run(self, settings: BuildSettings) -> Result<JoinHandle<Result<()>>> {
        if self.platform.is_mac_os() {
            return self.run_as_macos_app(settings).await;
        } else {
            return self.run_with_simctl(settings).await;
        }
    }
}

/// MacOS Runner
impl Runner {
    pub async fn run_as_macos_app(self, settings: BuildSettings) -> Result<JoinHandle<Result<()>>> {
        let nvim = self.state.clients.get(&self.client.pid)?;
        let ref mut logger = nvim.new_logger("Run", &self.target, &self.direction);

        logger.log_title().await?;
        logger.open_win().await?;

        tokio::spawn(async move {
            let program = settings.path_to_output_binary()?;
            let mut stream = Process::new(&program).stream()?;

            tracing::debug!("Running binary {program:?}");

            use ProcessItem::*;
            // NOTE: This is required so when neovim exist this should also exit
            while let Some(update) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let state = state.lock().await;
                let nvim = state.clients.get(&self.client.pid)?;
                let mut logger = nvim.new_logger("Run", &self.target, &self.direction);

                // NOTE: NSLog get directed to error by default which is odd
                match update {
                    Output(msg) => logger.log(msg).await?,
                    Error(msg) => logger.log(format!("[Error] {msg}")).await?,
                    Exit(ref code) => {
                        let success = code == "0";
                        let msg = fmt::as_section(if success {
                            "".into()
                        } else {
                            format!("Panic {code}")
                        });
                        logger.log(msg).await?;
                        logger.set_status_end(success, true).await?;
                    }
                }
            }
            Ok(())
        })
        .pipe(Ok)
    }
}

/// Simctl Runner
impl Runner {
    pub async fn run_with_simctl(self, settings: BuildSettings) -> Result<JoinHandle<Result<()>>> {
        let Self {
            client,
            target,
            state,
            udid,
            ..
        } = self;

        let ref mut logger = state.clients.get(&client.pid)?.new_unamed_logger();
        let app_id = settings.product_bundle_identifier;
        let path_to_app = settings.metal_library_output_dir;

        logger.set_running().await?;

        let runner = {
            let device = if let Some(udid) = udid {
                state.devices.iter().find(|d| d.udid == udid).cloned()
            } else {
                None
            }
            .ok_or_else(|| Error::Run("udid not found!!".to_string()))?;

            let runner = SimDeviceRunner::new(device, target.clone(), app_id, path_to_app);
            runner.boot(logger).await?;
            runner.install(logger).await?;
            runner
        };

        let mut launcher = runner.launch(logger).await?;
        let mut stream = launcher.stream()?;
        // TODO(daemon): ensure Simulator.app is running

        logger.log(fmt::separator()).await?;

        tokio::spawn(async move {
            while let Some(output) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let state = state.lock().await;
                let mut logger = match state.clients.get(&client.pid) {
                    Ok(nvim) => nvim.new_unamed_logger(),
                    Err(_) => {
                        tracing::info!("Nvim Instance closed, closing runner ..");
                        launcher.kill().await;
                        break;
                    }
                };

                use ProcessItem::*;
                match output {
                    Output(msg) => {
                        if !msg.contains("ignoring singular matrix") {
                            logger.log(msg).await?;
                        }
                    }
                    Error(msg) => {
                        logger.log(format!("[Error] {msg}")).await?;
                    }
                    Exit(code) => {
                        logger.log(format!("[Exit] {code}")).await?;
                        break;
                    }
                };
                drop(state);
            }

            drop(stream);

            Ok(())
        })
        .pipe(Ok)
    }
}
