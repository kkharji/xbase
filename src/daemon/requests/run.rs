use super::*;
use crate::{
    nvim::BufferDirection,
    types::{BuildConfiguration, Platform},
};

#[cfg(feature = "daemon")]
use {
    crate::{constants::DAEMON_STATE, types::SimDevice, xcode::append_build_root, Error},
    std::str::FromStr,
    tokio_stream::StreamExt,
    xcodebuild::runner,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceLookup {
    name: String,
    udid: String,
    platform: Platform,
}

/// Run a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub client: Client,
    pub config: BuildConfiguration,
    pub device: Option<DeviceLookup>,
    pub direction: Option<BufferDirection>,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for Run {
    async fn handle(self) -> Result<()> {
        tracing::info!("⚙️ Running command: {}", self.config.to_string());
        tracing::trace!("{:#?}", self);

        let Self { config, device, .. } = self;
        let Client { pid, root } = self.client;

        let direction = self.direction.clone();
        let state = DAEMON_STATE.clone().lock_owned().await;
        let platform = if let Some(d) = device.as_ref() {
            Some(d.platform.clone())
        } else {
            None
        };

        let nvim = state.clients.get(&pid)?;

        let args = {
            let mut args = config.as_args();
            if let Some(ref platform) = platform {
                args.extend(platform.sdk_simulator_args())
            }

            append_build_root(&root, args)?
        };

        tracing::debug!("args: {:?}", args);

        let settings = runner::build_settings(&root, &args).await?;
        let platform = platform.unwrap_or(Platform::from_str(&settings.platform_display_name)?);

        let (success, ref win) = nvim
            .new_logger("Build", &config.target, &direction)
            .log_build_stream(&root, &args, true, true)
            .await?;

        if !success {
            let msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        let mut logger = nvim.new_logger("Run", &config.target, &direction);

        if platform.is_mac_os() {
            let program = settings.path_to_output_binary()?;
            tracing::debug!("Running binary {program:?}");

            logger.log_title().await?;
            tokio::spawn(async move {
                let mut stream = runner::run(program).await?;

                use xcodebuild::runner::ProcessUpdate::*;
                // NOTE: This is required so when neovim exist this should also exit
                while let Some(update) = stream.next().await {
                    let state = DAEMON_STATE.clone();
                    let state = state.lock().await;

                    let nvim = state.clients.get(&pid)?;
                    let mut logger = nvim.new_logger("Run", &config.target, &direction);
                    let ref win = Some(logger.open_win().await?);

                    // NOTE: NSLog get directed to error by default which is odd
                    match update {
                        Stdout(msg) => {
                            logger.log(format!("[Output] {msg}"), win).await?;
                        }
                        Error(msg) | Stderr(msg) => {
                            logger.log(format!("[Error]  {msg}"), win).await?;
                        }
                        Exit(ref code) => {
                            logger.log(format!("[Exit]   {code}"), win).await?;
                            logger.set_status_end(code == "0", win.is_none()).await?;
                        }
                    }
                }
                anyhow::Ok(())
            });
            return Ok(());
        } else if let Some(mut device) = get_device(&state, device) {
            let path_to_app = settings.metal_library_output_dir;
            let app_id = settings.product_bundle_identifier;

            tracing::debug!("{app_id}: {:?}", path_to_app);

            logger.log_title().await?;
            tokio::spawn(async move {
                // NOTE: This is required so when neovim exist this should also exit
                let state = DAEMON_STATE.clone().lock_owned().await;
                let nvim = state.clients.get(&pid)?;
                let ref mut logger = nvim.new_logger("Run", &config.target, &direction);
                let ref win = Some(logger.open_win().await?);

                logger.set_running().await?;

                device.try_boot(logger, win).await?;
                device
                    .try_install(&path_to_app, &app_id, logger, win)
                    .await?;
                device.try_launch(&app_id, logger, win).await?;

                // TODO: Remove and repalce with app logs
                logger.set_status_end(true, win.is_none()).await?;

                let mut state = DAEMON_STATE.clone().lock_owned().await;
                state.devices.insert(device);
                state.sync_client_state().await
            });
            return Ok(());
        }

        let msg = format!("Unable to run `{}` under `{platform}`", config.target);
        logger.log(msg.clone(), win).await?;
        Err(Error::Run(msg))
    }
}

// let target = project.get_target(&config.target, ,)?;
#[cfg(feature = "daemon")]
fn get_device<'a>(
    state: &'a tokio::sync::OwnedMutexGuard<crate::state::State>,
    device: Option<DeviceLookup>,
) -> Option<SimDevice> {
    if let Some(device) = device {
        state
            .devices
            .iter()
            .find(|d| d.name == device.name && d.udid == device.udid)
            .cloned()
    } else {
        None
    }
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Run> for Run {
    fn pre(lua: &Lua, msg: &Run) -> LuaResult<()> {
        lua.print(&msg.to_string());
        Ok(())
    }
}

impl ToString for Run {
    fn to_string(&self) -> String {
        if let Some(ref device) = self.device {
            format!("run [{}] with {}", device.name, self.config.to_string())
        } else {
            format!("run with {}", self.config.to_string())
        }
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Run {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        let table = match lua_value {
            LuaValue::Table(t) => Ok(t),
            _ => Err(LuaError::external("Fail to deserialize Run")),
        }?;

        let device: Option<LuaTable> = table.get("device")?;

        Ok(Self {
            client: table.get("client")?,
            config: table.get("config")?,
            direction: table.get("direction").ok(),
            device: device
                .map(|d| {
                    let name = d.get("name").ok()?;
                    let udid = d.get("udid").ok()?;
                    let platform = d.get("platform").ok()?;
                    Some(DeviceLookup {
                        name,
                        udid,
                        platform,
                    })
                })
                .flatten(),
        })
    }
}
