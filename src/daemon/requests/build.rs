#[cfg(feature = "mlua")]
use crate::daemon::Daemon;

#[cfg(feature = "daemon")]
use crate::daemon::nvim::WindowType;

#[cfg(feature = "daemon")]
use async_stream::stream;

#[cfg(feature = "daemon")]
use tokio_stream::StreamExt;

#[cfg(feature = "daemon")]
use tap::Pipe;

#[cfg(feature = "daemon")]
use crate::daemon::{DaemonRequestHandler, DaemonState};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Build a project.
#[derive(Debug)]
pub struct Build {
    pub pid: i32,
    pub root: String,
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

impl Build {
    pub const KEY: &'static str = "build";
}

// TODO: Implement build command
// On neovim side:
// - Call the command after picking the target. If their is only a single target then just use that
//  - This requires somehow given the client all information it needs in order present the user
//  with the options needed to build
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
impl DaemonRequestHandler<Build> for Build {
    fn parse(args: Vec<&str>) -> Result<Self> {
        if let (Some(pid), Some(root)) = (args.get(0), args.get(1)) {
            Ok(Self {
                pid: pid.parse::<i32>()?,
                root: root.to_string(),
                target: args.get(2).map(ToString::to_string),
                configuration: args.get(3).map(ToString::to_string),
                scheme: args.get(4).map(ToString::to_string),
            })
        } else {
            anyhow::bail!("Missing arugments: {:?}", args)
        }
    }

    async fn handle(&self, state: DaemonState) -> Result<()> {
        tracing::debug!("Handling build request..");
        let state = state.lock().await;
        let ws = match state.workspaces.get(&self.root) {
            Some(ws) => ws,
            None => anyhow::bail!("No workspace for {}", self.root),
        };

        ws.project
            .xcodebuild(&["build"])
            .await?
            .pipe(|mut logs| {
                stream! {
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
                }
            })
            .pipe(Box::pin)
            .pipe(|stream| async {
                let nvim = match ws.clients.get(&self.pid) {
                    Some(nvim) => nvim,
                    None => anyhow::bail!("No nvim client found to build project."),
                };
                nvim.log_to_buffer("Build", WindowType::Vertical, stream, true)
                    .await
            })
            .await?;

        Ok(())
    }
}

#[cfg(feature = "lua")]
impl Build {
    pub fn lua(
        lua: &mlua::Lua,
        (pid, root, t, c, s): (i32, String, Option<String>, Option<String>, Option<String>),
    ) -> mlua::Result<()> {
        use crate::util::mlua::LuaExtension;
        lua.trace(
            format!(
                "Build (target: {:?} configuration: {:?}, scheme: {:?})",
                t, c, s
            )
            .as_ref(),
        )?;

        let mut args = vec!["build".into(), pid.to_string(), root];

        if let Some(target) = t {
            args.push(target)
        }
        if let Some(configuration) = c {
            args.push(configuration)
        }
        if let Some(scheme) = s {
            args.push(scheme)
        }

        Daemon::execute(&args.join(" ").split(" ").collect::<Vec<&str>>())
    }
}
