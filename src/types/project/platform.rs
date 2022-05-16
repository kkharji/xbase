use super::*;
#[derive(Debug, Default, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum Platform {
    #[serde(rename = "iOS")]
    IOS,
    #[serde(rename = "watchOS")]
    WatchOS,
    #[serde(rename = "tvOS")]
    TvOS,
    #[serde(rename = "macOS")]
    MacOS,
    #[default]
    None,
}

impl Platform {
    /// Returns `true` if the platfrom is [`IOS`].
    ///
    /// [`IOS`]: Platfrom::IOS
    #[must_use]
    pub fn is_ios(&self) -> bool {
        matches!(self, Self::IOS)
    }

    /// Returns `true` if the platfrom is [`WatchOS`].
    ///
    /// [`WatchOS`]: Platfrom::WatchOS
    #[must_use]
    pub fn is_watch_os(&self) -> bool {
        matches!(self, Self::WatchOS)
    }

    /// Returns `true` if the platfrom is [`TvOS`].
    ///
    /// [`TvOS`]: Platfrom::TvOS
    #[must_use]
    pub fn is_tv_os(&self) -> bool {
        matches!(self, Self::TvOS)
    }

    /// Returns `true` if the platfrom is [`MacOS`].
    ///
    /// [`MacOS`]: Platfrom::MacOS
    #[must_use]
    pub fn is_mac_os(&self) -> bool {
        matches!(self, Self::MacOS)
    }
}
