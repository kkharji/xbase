use anyhow::anyhow as err;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use simctl::list::DeviceState;
use simctl::Device;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use crate::nvim::Logger;
use crate::nvim::NvimWindow;
use crate::util::string_as_section;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimDevice {
    pub is_running: bool,
    #[serde(flatten)]
    inner: Device,
}

impl Eq for SimDevice {}

impl PartialEq for SimDevice {
    fn eq(&self, other: &Self) -> bool {
        self.inner.udid == other.inner.udid
    }
}

impl Hash for SimDevice {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.udid.hash(state)
    }
}

impl Deref for SimDevice {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SimDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(feature = "daemon")]
impl SimDevice {
    pub fn new(inner: Device) -> Self {
        Self {
            inner,
            is_running: false,
        }
    }

    pub async fn try_boot<'a>(
        &mut self,
        logger: &mut Logger<'a>,
        win: &Option<NvimWindow>,
    ) -> Result<()> {
        if let DeviceState::Shutdown = &self.state {
            self.boot().map_err(|e| err!("{:#?}", e))?;
            self.state = DeviceState::Booted;

            let msg = format!("[Booting] \"{}\"", self.name);
            tracing::info!("{msg}");
            logger.log(msg, win).await?;
        }

        Ok(())
    }

    pub async fn try_install<'a>(
        &mut self,
        path_to_app: &PathBuf,
        app_id: &String,
        logger: &mut Logger<'a>,
        win: &Option<NvimWindow>,
    ) -> Result<()> {
        self.install(path_to_app).map_err(|e| err!("{:#?}", e))?;
        let msg = format!("[Installed] \"{}\" {app_id}", self.name);
        tracing::info!("{msg}");
        logger.log(msg, win).await?;
        Ok(())
    }

    pub async fn try_launch<'a>(
        &mut self,
        app_id: &String,
        logger: &mut Logger<'a>,
        win: &Option<NvimWindow>,
    ) -> Result<()> {
        if !self.is_running {
            tracing::info!("[Launching] \"{}\" {app_id}", self.name);
            self.launch(app_id)
                .stdout(&"/tmp/wordle_log")
                .stderr(&"/tmp/wordle_log")
                .exec()
                .map_err(|e| err!("{:#?}", e))?;

            self.is_running = true;

            tracing::info!("[Launched]");
            logger
                .log(string_as_section("[Launched]".into()), win)
                .await?;
        }

        Ok(())
    }
}
