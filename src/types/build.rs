use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display as EnumDisplay, EnumString};

#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
// #[serde(untagged)]
pub enum XConfiguration {
    Debug,
    Release,
    Custom(String),
}

/// Fields required to build a project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildConfiguration {
    /// TODO(nvim): make build config sysroot default to tmp in auto-build
    pub sysroot: Option<String>,
    /// Target to build
    pub target: String,
    /// Configuration to build with, default Debug
    pub configuration: XConfiguration,
    /// Scheme to build with
    pub scheme: Option<String>,
}

impl Display for BuildConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xcodebuild")?;
        write!(f, " -configuration {}", self.configuration)?;

        if let Some(ref sysroot) = self.sysroot {
            write!(f, " -sysroot {sysroot}")?;
        }
        if let Some(ref scheme) = self.scheme {
            write!(f, " -scheme {scheme}")?;
        }
        write!(f, " -target {}", self.target)?;
        Ok(())
    }
}

impl BuildConfiguration {
    pub fn as_args(&self) -> Vec<String> {
        let mut args = self
            .to_string()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        args.remove(0);
        args.insert(0, "build".to_string());
        args
    }
}

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for BuildConfiguration {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        use tap::Pipe;

        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                sysroot: table.get("sysroot")?,
                target: table.get("target")?,
                configuration: table
                    .get::<_, String>("configuration")?
                    .pipe(|s| XConfiguration::from_str(&s))
                    .to_lua_err()?,
                scheme: table.get("scheme")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize BuildConfiguration"))
        }
    }
}
