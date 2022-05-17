use super::*;
use crate::{
    nvim::BufferDirection,
    types::{BuildConfiguration, Platform},
};

#[cfg(feature = "daemon")]
use {
    crate::constants::DAEMON_STATE, crate::types::SimDevice, anyhow::anyhow as err,
    xcodebuild::runner::build_settings,
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

        let nvim = state
            .clients
            .get(&pid)
            .ok_or_else(|| err!("no client found with {}", pid))?;

        let args = {
            let mut args = config.as_args();
            if let Some(platform) = platform {
                args.extend(platform.sdk_simulator_args())
            }
            args
        };

        let build_settings = build_settings(&root, &args).await?;
        let ref app_id = build_settings.product_bundle_identifier;

        // FIX(run): When running with release path_to_app is incorrect
        //
        // Err: application bundle was not found at the provided path.\nProvide a valid path to the
        // desired application bundle.
        //
        // Path doesn't point to local directory build
        let ref path_to_app = build_settings.metal_library_output_dir;

        tracing::debug!("{app_id}: {:?}", path_to_app);
        let (success, ref win) = nvim
            .new_logger("Build", &config.target, &direction)
            .log_build_stream(&root, &args, false, true)
            .await?;

        if !success {
            let msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(&msg).await?;
            anyhow::bail!("{msg}");
        }

        let ref mut logger = nvim.new_logger("Run", &config.target, &direction);

        logger.set_running().await?;

        if let Some(mut device) = get_device(&state, device) {
            device.try_boot(logger, win).await?;
            device.try_install(path_to_app, app_id, logger, win).await?;
            device.try_launch(app_id, logger, win).await?;

            logger.set_status_end(true, true).await?;

            tokio::spawn(async move {
                let mut state = DAEMON_STATE.clone().lock_owned().await;
                state.devices.insert(device);
                state.sync_client_state().await
            });
        } else {
            // TODO: check if macOS is the platform and run it
        }

        Ok(())
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
