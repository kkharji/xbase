use super::*;
use crate::watch::Event;
use crate::{Error, Result};
use futures::StreamExt;
use process_stream::{Process, ProcessItem};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use tokio::process::Command;
use xbase_proto::Client;
use xcodeproj::pbxproj::PBXTargetPlatform;

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct SwiftProject {
    name: String,
    root: PathBuf,
    targets: HashMap<String, PBXTargetPlatform>,
    clients: Vec<i32>,
    watchignore: Vec<String>,
}

impl ProjectData for SwiftProject {
    fn root(&self) -> &PathBuf {
        &self.root
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn targets(&self) -> &HashMap<String, PBXTargetPlatform> {
        &self.targets
    }

    fn clients(&self) -> &Vec<i32> {
        &self.clients
    }

    fn clients_mut(&mut self) -> &mut Vec<i32> {
        &mut self.clients
    }

    fn watchignore(&self) -> &Vec<String> {
        &self.watchignore
    }
}

#[async_trait::async_trait]
impl ProjectBuild for SwiftProject {
    fn build(
        &self,
        cfg: &BuildSettings,
        _device: Option<&Device>,
    ) -> Result<(StringStream, Vec<String>)> {
        log::info!("Building {}", cfg.target);

        let args = vec!["build", "--target", &cfg.target];
        let mut process = Process::new("/usr/bin/swift");

        process.args(&args);
        process.current_dir(self.root());

        let mut stream = process.spawn_and_stream()?;
        let stream = stream! {
            while let Some(output) =  stream.next().await {
                if let ProcessItem::Exit(v) = output {
                    if !v.eq("0") {
                        yield String::from("FAILED")
                    }
                } else {
                    log::trace!("{output}");
                    yield output.to_string()
                }

            }
        };
        Ok((stream.boxed(), vec![]))
    }
}

#[async_trait::async_trait]
impl ProjectRun for SwiftProject {
    fn get_runner(
        &self,
        cfg: &BuildSettings,
        _device: Option<&Device>,
    ) -> Result<(Box<dyn Runner + Send + Sync>, StringStream)> {
        let (build_stream, _) = self.build(cfg, None)?;

        let output = std::process::Command::new("/usr/bin/swift")
            .args(["build", "--show-bin-path"])
            .current_dir(self.root())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();
            return Err(Error::Run(format!(
                "Getting target bin path failed {stderr}"
            )));
        }

        // WARN: THIS MIGHT FAIL BECAUSE BUILD IS NOT YET RAN
        let output = String::from_utf8(output.stdout).unwrap();
        let bin_path = PathBuf::from(output.trim()).join(&cfg.target);

        log::info!("Running {:?} via {bin_path:?}", self.name());

        Ok((Box::new(BinRunner::from_path(&bin_path)), build_stream))
    }
}

#[async_trait::async_trait]
impl ProjectCompile for SwiftProject {
    async fn update_compile_database(&self) -> Result<()> {
        // No Compile database needed for swif projects
        Ok(())
    }
}
#[async_trait::async_trait]
impl ProjectGenerate for SwiftProject {
    fn should_generate(&self, event: &Event) -> bool {
        let is_config_file = event.file_name() == "Package.swift";
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update
            || event.is_create_event()
            || event.is_remove_event()
            || event.is_rename_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self) -> Result<()> {
        log::info!("Building and compiling swift project {}", self.name());

        let mut process: Process = vec!["/usr/bin/swift", "build"].into();

        process.current_dir(self.root());

        let (success, logs) = consume_and_log(process.spawn_and_stream()?.boxed()).await;

        if !success {
            return Err(Error::SwiftBuildForComplilation(
                self.name().into(),
                logs.join("\n"),
            ));
        }

        self.update_project_info().await?;

        log::info!("(name: {:?}, targets: {:?})", self.name(), self.targets());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for SwiftProject {
    async fn new(client: &Client) -> Result<Self> {
        let Client { root, pid, .. } = client;

        let watchignore = generate_watchignore(root).await;

        let mut project = Self {
            root: root.clone(),
            watchignore,
            clients: vec![pid.clone()],
            ..Self::default()
        };

        if !root.join(".build").exists() {
            log::info!("no .build directory found at {root:?}");
            project.generate().await?;
            return Ok(project);
        } else {
            project.update_project_info().await?;
            log::info!(
                "(name: {:?}, targets: {:?})",
                project.name(),
                project.targets()
            );
        }

        Ok(project)
    }
}

impl SwiftProject {
    /// Read Package.swift and update internal state
    async fn update_project_info(&mut self) -> Result<()> {
        use anyhow::anyhow;
        use serde_json::{Map, Value};

        let output = Command::new("/usr/bin/swift")
            .args(["package", "dump-package"])
            .current_dir(self.root())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;

        let map = if output.status.success() {
            serde_json::from_slice::<Map<String, Value>>(&output.stdout)?
        } else {
            let error = String::from_utf8(output.stderr)
                .unwrap_or_default()
                .split("\n")
                .collect();
            log::error!("Fail to read swift package information {error}");
            return Err(Error::SwiftPackageRead(self.name().into(), error));
        };

        // TODO(swift-package): only provide run service for executables
        self.name = map
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("expected package name field is missing!"))?;

        self.targets = map
            .get("targets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("expected package target field is missing!"))?
            .into_iter()
            .flat_map(|v| v.as_object())
            .flat_map(|target_info| {
                let name = target_info.get("name")?.as_str()?.to_string();
                if !target_info
                    .get("type")
                    .and_then(|s| s.as_str())
                    .map(|s| s == "test")
                    .unwrap_or_default()
                {
                    Some((name, PBXTargetPlatform::MacOS))
                } else {
                    None
                }
            })
            .collect();

        Ok(())
    }
}
