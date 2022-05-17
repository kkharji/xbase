use anyhow::Result;
use serde::{Deserialize, Serialize};
use simctl::list::DeviceState;
use simctl::Device;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tap::Pipe;

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
            logger
                .log(format!("[Booting] ({})", self.name), win)
                .await?;

            self.boot()
                .pipe(|res| self.handle_error(res, logger, win))
                .await?;
            self.state = DeviceState::Booted;
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
        self.install(path_to_app)
            .pipe(|res| self.handle_error(res, logger, win))
            .await?;
        logger
            .log(format!("[Installing] ({}) {app_id}", self.name), win)
            .await
    }

    pub async fn try_launch<'a>(
        &mut self,
        app_id: &String,
        logger: &mut Logger<'a>,
        win: &Option<NvimWindow>,
    ) -> Result<()> {
        if !self.is_running {
            logger
                .log(format!("[Launching] ({}) {app_id}", self.name), win)
                .await?;

            self.launch(app_id)
                .stdout(&"/tmp/wordle_log")
                .stderr(&"/tmp/wordle_log")
                .exec()
                .pipe(|res| self.handle_error(res, logger, win))
                .await?;

            self.is_running = true;

            logger
                .log(string_as_section("[Launched]".into()), win)
                .await?;
        }

        Ok(())
    }

    async fn handle_error<'a, T>(
        &mut self,
        res: simctl::Result<T>,
        logger: &mut Logger<'a>,
        win: &Option<NvimWindow>,
    ) -> Result<()> {
        if let Err(e) = to_anyhow_error(res) {
            logger.log(e.to_string(), win).await?;
            logger.set_status_end(false, true).await?;
            self.is_running = false;
        }
        Ok(())
    }
}

fn to_anyhow_error<T>(v: simctl::Result<T>) -> Result<T> {
    v.map_err(|e| match e {
        simctl::Error::Output { stderr, .. } => {
            anyhow::anyhow!("External Command Failure: {stderr}")
        }
        simctl::Error::Io(err) => anyhow::Error::new(err),
        simctl::Error::Json(err) => anyhow::Error::new(err),
        simctl::Error::Utf8(err) => anyhow::Error::new(err),
    })
}
