use super::*;
#[derive(Clone, Debug, Default, Deserialize, Serialize, Hash, PartialEq, Eq)]
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
    pub fn sdk_simulator_args(&self) -> Vec<String> {
        match self {
            Platform::IOS => vec!["-sdk".into(), "iphonesimulator".into()],
            Platform::WatchOS => vec!["-sdk".into(), "watchsimulator".into()],
            Platform::TvOS => vec!["-sdk".into(), "appletvsimulator".into()],
            Platform::MacOS => vec!["-sdk".into(), "macosx".into()],
            Platform::None => vec![],
        }
        // -sdk driverkit -sdk iphoneos -sdk macosx -sdk appletvos -sdk watchos
    }

    #[must_use]
    pub fn is_ios(&self) -> bool {
        matches!(self, Self::IOS)
    }
    #[must_use]
    pub fn is_watch_os(&self) -> bool {
        matches!(self, Self::WatchOS)
    }
    #[must_use]
    pub fn is_tv_os(&self) -> bool {
        matches!(self, Self::TvOS)
    }
    #[must_use]
    pub fn is_mac_os(&self) -> bool {
        matches!(self, Self::MacOS)
    }
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match self {
            Platform::IOS => "iOS",
            Platform::WatchOS => "watchOS",
            Platform::TvOS => "tvOS",
            Platform::MacOS => "macOS",
            Platform::None => "",
        }
        .into()
    }
}

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for Platform {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::String(value) = lua_value {
            match value.to_str()? {
                "iOS" => Platform::IOS,
                "watchOS" => Platform::WatchOS,
                "tvOS" => Platform::TvOS,
                "macOS" => Platform::MacOS,
                _ => return Err(LuaError::external("Fail to deserialize Platform")),
            }
            .pipe(Ok)
        } else {
            Err(LuaError::external(
                "Fail to deserialize Platform, expected string",
            ))
        }
    }
}
