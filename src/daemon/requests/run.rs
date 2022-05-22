use {
    super::*, crate::nvim::BufferDirection, crate::store::DeviceLookup,
    crate::types::BuildConfiguration,
};

#[cfg(feature = "daemon")]
use {
    crate::constants::DAEMON_STATE,
    crate::runner::Runner,
    crate::types::Platform,
    crate::util::serde::value_or_default,
    crate::xcode::{append_build_root, build_with_loggger},
    crate::Error,
    xcodebuild::runner::build_settings,
};

/// Run a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub client: Client,
    pub config: BuildConfiguration,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub device: DeviceLookup,
    #[cfg_attr(feature = "daemon", serde(deserialize_with = "value_or_default"))]
    pub direction: BufferDirection,
}

#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl Handler for Run {
    async fn handle(self) -> Result<()> {
        let Client { pid, root, .. } = &self.client;
        tracing::info!("{:#?}", self);

        tracing::info!("⚙️ Running command: {}", self.config.to_string());

        let state = DAEMON_STATE.clone().lock_owned().await;
        let device = state.devices.from_lookup(self.device);
        tracing::info!("{:#?}", device);

        let nvim = state.clients.get(&pid)?;
        let args = {
            let mut args = self.config.as_args();
            if let Some(ref device) = device {
                args.extend(device.special_build_args())
            }
            append_build_root(&root, args)?
        };

        let ref mut logger = nvim.logger();

        logger.set_title(format!("Run:{}", self.config.target));
        logger.set_direction(&self.direction);

        let settings = build_settings(&root, &args).await?;
        let platform = device
            .as_ref()
            .map(|d| d.platform.clone())
            .unwrap_or_else(|| Platform::from_display(&settings.platform_display_name).unwrap());

        let success = build_with_loggger(logger, &root, &args, true, true).await?;
        if !success {
            let msg = format!("Failed: {} ", self.config.to_string());
            nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        // TODO(daemon): insert handler to state.runners
        // TODO(nvim): provide mapping to close runners.
        //
        // If there is more then one runner then pick, else close from current buffer.
        // C-c in normal/insert mode should close that process
        Runner {
            target: self.config.target,
            platform,
            client: self.client,
            state,
            args,
            udid: device.map(|d| d.udid.clone()),
            direction: self.direction,
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
        if let Some(ref name) = self.device.name {
            format!("run [{}] with {}", name, self.config.to_string())
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
            direction: table.get("direction").unwrap_or_default(),
            device: device
                .map(|d| {
                    let name = d.get("name").ok()?;
                    let udid = d.get("udid").ok()?;
                    Some(DeviceLookup { name, udid })
                })
                .flatten()
                .unwrap_or_default(),
        })
    }
}
