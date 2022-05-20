mod simctl;
pub use self::simctl::*;

use crate::{
    constants::DAEMON_STATE,
    nvim::BufferDirection,
    state::State,
    types::{Client, Platform, SimDevice},
    util::string_as_section,
    Error, Result,
};
use {
    tap::Pipe,
    tokio::{sync::OwnedMutexGuard, task::JoinHandle},
    tokio_stream::StreamExt,
    xcodebuild::{parser::BuildSettings, runner},
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
            let mut stream = runner::run(&program).await?;

            tracing::debug!("Running binary {program:?}");

            use xcodebuild::runner::ProcessUpdate::*;
            // NOTE: This is required so when neovim exist this should also exit
            while let Some(update) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let state = state.lock().await;
                let nvim = state.clients.get(&self.client.pid)?;
                let mut logger = nvim.new_logger("Run", &self.target, &self.direction);

                // NOTE: NSLog get directed to error by default which is odd
                match update {
                    Stdout(msg) => {
                        logger.log(msg).await?;
                    }
                    Error(msg) | Stderr(msg) => {
                        logger.log(format!("[Error]  {msg}")).await?;
                    }
                    Exit(ref code) => {
                        let success = code == "0";
                        let msg = string_as_section(if success {
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
            direction,
            ..
        } = self;

        let ref mut logger = state
            .clients
            .get(&client.pid)?
            .new_logger("Run", &target, &direction);
        let app_id = settings.product_bundle_identifier;
        let path_to_app = settings.metal_library_output_dir;

        logger.set_running().await?;
        let _runner = {
            let device = get_device(&state, udid)?;
            let runner = SimDeviceRunner::new(device, target.clone(), app_id, path_to_app);
            runner.boot(logger).await?;
            runner.install(logger).await?;
            runner.launch(logger).await?;
            runner
        };

        // NOTE: This is required so when neovim exist this should also exit
        tokio::spawn(async move {
            let state = DAEMON_STATE.clone().lock_owned().await;
            let nvim = state.clients.get(&client.pid)?;
            let ref mut _logger = nvim.new_logger("Run", &target, &direction);

            state.sync_client_state().await?;

            Ok(())
        })
        .pipe(Ok)
    }
}

fn get_device<'a>(state: &'a OwnedMutexGuard<State>, udid: Option<String>) -> Result<SimDevice> {
    if let Some(udid) = udid {
        state.devices.iter().find(|d| d.udid == udid).cloned()
    } else {
        None
    }
    .ok_or_else(|| Error::Run("udid not found!!".to_string()))
}
