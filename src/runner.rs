use crate::constants::DAEMON_STATE;
use crate::util::string_as_section;
use crate::{
    nvim::BufferDirection,
    state::State,
    types::{Client, Platform},
    Result,
};
use tap::Pipe;
use tokio::sync::OwnedMutexGuard;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use xcodebuild::parser::BuildSettings;
use xcodebuild::runner;

mod macos;
mod simctl;

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
