use super::*;
use crate::types::BuildConfiguration;
use std::fmt::Debug;

/// Build a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub client: Client,
    pub config: BuildConfiguration,
}

#[cfg(feature = "lua")]
impl<'a> Requester<'a, Build> for Build {
    fn pre(lua: &Lua, msg: &Build) -> LuaResult<()> {
        lua.print(&format!("{}", msg.config.to_string()));
        Ok(())
    }
}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Build {
    async fn handle(self, state: DaemonState) -> Result<()> {
        tracing::debug!("Handling build request..");

        let state = state.lock().await;
        let ws = state.get_workspace(&self.client.root)?;
        let nvim = ws.get_client(&self.client.pid)?;
        let mut logs = ws.project.xcodebuild(&["build"], self.config).await?;
        let stream = Box::pin(stream! {
            use xcodebuild::parser::Step::*;
            while let Some(step) = logs.next().await {
                let line = match step {
                    Exit(_) => { continue; }
                    BuildSucceed | CleanSucceed | TestSucceed | TestFailed | BuildFailed => {
                        format! {
                            "{} ----------------------------------------------------",
                            step.to_string().trim().to_string()
                        }
                    }
                    step => step.to_string().trim().to_string(),
                };
                if !line.is_empty() {
                    for line in line.split("\n") {
                        yield line.to_string();
                    }
                }
            }
        });

        nvim.log_to_buffer("Build", None, stream, true).await?;

        Ok(())
    }
}

#[cfg(feature = "mlua")]
impl<'a> FromLua<'a> for Build {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                client: table.get("client")?,
                config: table.get("config")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Build"))
        }
    }
}
