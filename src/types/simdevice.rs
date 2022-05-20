use crate::error::Error;
use crate::Result;
use serde::{Deserialize, Serialize};
use simctl::list::DeviceState;
use simctl::Device;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tap::Pipe;

use crate::nvim::Logger;
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

    pub async fn try_boot<'a>(&mut self, logger: &mut Logger<'a>) -> Result<()> {
        // FIX(run): DeviceState can get out of sync when the user shutdown device manually
        logger
            .log(string_as_section(format!("Booting:    {}", self.name)))
            .await?;

        if let Err(e) = self.boot() {
            let err = Error::from(e);
            let err_msg = err.to_string();
            if !err_msg.contains("current state Booted") {
                logger.log(err_msg).await?;
                logger.set_status_end(false, true).await?;
                self.is_running = false;
                return Err(err);
            }
        };

        self.state = DeviceState::Booted;

        Ok(())
    }

    pub async fn try_install<'a>(
        &mut self,
        path_to_app: &PathBuf,
        app_id: &String,
        logger: &mut Logger<'a>,
    ) -> Result<()> {
        logger
            .log(string_as_section(format!("Installing: {app_id}",)))
            .await?;
        self.install(path_to_app)
            .pipe(|res| self.handle_error(res, logger))
            .await?;
        Ok(())
    }

    pub async fn try_launch<'a>(&mut self, app_id: &String, logger: &mut Logger<'a>) -> Result<()> {
        logger
            .log(string_as_section(format!("Launching:  {app_id}")))
            .await?;
        if !self.is_running {
            self.launch(app_id)
                .stdout(&"/tmp/wordle_log")
                .stderr(&"/tmp/wordle_log")
                .exec()
                .pipe(|res| self.handle_error(res, logger))
                .await?;

            self.is_running = true;

            logger.log(string_as_section("Connected".into())).await?;
        }

        Ok(())
    }

    async fn handle_error<'a, T>(
        &mut self,
        res: simctl::Result<T>,
        logger: &mut Logger<'a>,
    ) -> Result<()> {
        if let Err(e) = res {
            let err = Error::from(e);
            logger.log(err.to_string()).await?;
            logger.set_status_end(false, true).await?;
            self.is_running = false;
            return Err(err);
        }
        Ok(())
    }
}
