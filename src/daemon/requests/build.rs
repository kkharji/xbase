use super::*;
use std::fmt::Debug;

/// Build a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub pid: i32,
    pub root: String,
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

#[cfg(feature = "lua")]
impl<'a> Requestor<'a, Build> for Build {}

#[cfg(feature = "daemon")]
#[async_trait]
impl Handler for Build {
    async fn handle(&self, state: DaemonState) -> Result<()> {
        tracing::debug!("Handling build request..");

        let state = state.lock().await;
        let ws = state.get_workspace(&self.root)?;
        let nvim = ws.get_client(&self.pid)?;
        let mut logs = ws.project.xcodebuild(&["build"]).await?;
        let stream = Box::pin(stream! {
            while let Some(step) = logs.next().await {
                let line = match step {
                    xcodebuild::parser::Step::Exit(_) => { continue; }
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
                pid: table.get("pid")?,
                root: table.get("root")?,
                target: table.get("target")?,
                configuration: table.get("configuration")?,
                scheme: table.get("scheme")?,
            })
        } else {
            Err(LuaError::external("Fail to deserialize Build"))
        }
    }
}
