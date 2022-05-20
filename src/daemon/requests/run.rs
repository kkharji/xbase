use {
    super::*,
    crate::nvim::BufferDirection,
    crate::types::{BuildConfiguration, Platform},
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
use {
    crate::constants::DAEMON_STATE,
    crate::runner::Runner,
    crate::xcode::{append_build_root, build_with_loggger},
    crate::Error,
    xcodebuild::runner::build_settings,
};

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for Run {
    async fn handle(self) -> Result<()> {
        let Client { pid, root, .. } = &self.client;

        tracing::info!("⚙️ Running command: {:?}", self);
        tracing::info!("⚙️ Running command: {}", self.config.to_string());

        let state = DAEMON_STATE.clone().lock_owned().await;
        let direction = self.direction.clone();
        let platform = if let Some(ref d) = self.device {
            tracing::info!("{:#?}", d.platform);
            Some(d.platform.clone())
        } else {
            None
        };

        let nvim = state.clients.get(&pid)?;
        let args = {
            let mut args = self.config.as_args();
            if let Some(ref platform) = platform {
                args.extend(platform.sdk_simulator_args())
            }
            append_build_root(&root, args)?
        };

        let ref mut logger = nvim.new_logger("Build", &self.config.target, &direction);

        let settings = build_settings(&root, &args).await?;
        let platform = match platform {
            Some(v) => v,
            None => Platform::from_display(&settings.platform_display_name)?,
        };

        let success = build_with_loggger(logger, &root, &args, true, true).await?;
        if !success {
            let msg = format!("Failed: {} ", self.config.to_string());
            nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        Runner {
            target: self.config.target,
            platform,
            client: self.client,
            state,
            args,
            udid: self.device.map(|d| d.udid),
            direction,
        }
        .run(settings)
        .await?;

        Ok(())
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
