mod generator;

use crate::device::Device;
use crate::error::XcodeGenError;
use crate::watch::Event;
use crate::Error;
use crate::{state::State, Result};
use generator::ProjectGenerator;
use process_stream::{ProcessItem, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildSettings, Client};
use xcodeproj::pbxproj::PBXTargetPlatform;
use xcodeproj::XCodeProject;

/// Project Inner
#[derive(Debug)]
pub enum ProjectInner {
    None,
    XCodeProject(XCodeProject),
    Swift,
}

impl Default for ProjectInner {
    fn default() -> Self {
        Self::None
    }
}

/// Represent XcodeGen Project
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    pub name: String,

    /// The list of targets in the project mapped by name
    pub targets: HashMap<String, PBXTargetPlatform>,

    /// Root directory
    #[serde(skip)]
    pub root: PathBuf,

    /// Connected Clients
    #[serde(default)]
    pub clients: Vec<i32>,

    /// Ignore Patterns
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Generator
    pub generator: ProjectGenerator,

    #[serde(skip)]
    /// XCodeProject Data
    inner: ProjectInner,
}

impl Project {
    pub async fn new(client: &Client) -> Result<Self> {
        let Client { root, .. } = client;
        let mut project = Self::default();

        project.generator = ProjectGenerator::new(root);

        project.inner = if root.join("Package.swift").exists() {
            log::debug!("[Project] Kind: \"Swift\"",);
            // TODO(project): Get swift project name
            project.name = "UnknownSwiftProject".into();
            ProjectInner::Swift
        } else {
            let xcodeproj = match XCodeProject::new(root) {
                Ok(p) => p,
                Err(e) => {
                    log::info!("No XCodeProject found!");
                    // most likely xcodeproj haven't yet been generated. so we
                    if project.generator.is_none() {
                        log::error!("No Generator!!!");
                        return Err(e.into());
                    } else {
                        log::info!("Generating xcodeproj using {:?}...", project.generator);
                        if let Some(mut stream) = project.generator.regenerate(root).await? {
                            // let state = DAEMON_STATE.lock().await;
                            // let mut logger = state.clients.get(pid)?.logger();
                            // logger.open_win().await?;

                            // TODO(daemon): log xcodeproj generate to nvim buffer
                            //
                            // This requires reworking state
                            let mut success = true;
                            while let Some(output) = stream.next().await {
                                if output.is_exit() {
                                    success = output.as_exit().unwrap().eq("0");
                                    if !success {
                                        log::error!(
                                            "[ERROR] FAILED to generate xcodeproj using {:?}",
                                            project.generator
                                        )
                                        // logger.append("fail to generate compile commands").await?;
                                    };
                                } else {
                                    log::info!("{output}");
                                }
                            }
                            if !success {
                                return Err(Error::XcodeGen(XcodeGenError::XcodeProjUpdate(
                                    project.name.clone(),
                                )));
                            }
                        }
                        XCodeProject::new(root)?
                    }
                }
            };

            project.name = xcodeproj.name().to_owned();
            project.targets = xcodeproj.targets_platform();

            log::debug!("[New Project] name: {:?}", project.name);
            log::debug!("[New Project] Kind: \"XCodeProject\"");
            log::debug!("[New Project] Generator: \"{:?}\"", project.generator);
            log::debug!("[New Project] targets: {:?}", project.targets);

            ProjectInner::XCodeProject(xcodeproj)
        };

        project
            .ignore_patterns
            .extend(gitignore_to_glob_patterns(root).await?);

        project.ignore_patterns.extend(vec![
            "**/.git/**".into(),
            "**/.*".into(),
            "**/build/**".into(),
            "**/buildServer.json".into(),
        ]);

        Ok(project)
    }

    pub async fn update(&mut self, client: &Client) -> Result<()> {
        let Self { root, clients, .. } = self;
        let (clients, root) = (clients.clone(), root.clone());

        *self = Self::new(client).await?;

        self.root = root;
        self.clients = clients;
        log::info!("[Projects] update({:?})", self.name);

        Ok(())
    }

    pub async fn remove_target_watchers<'a>(
        &self,
        state: &'a mut MutexGuard<'_, State>,
    ) -> Result<()> {
        state.watcher.get_mut(&self.root)?.listeners.clear();
        Ok(())
    }

    pub async fn regenerate(
        &self,
        event: Option<&Event>,
    ) -> Result<Option<impl Stream<Item = ProcessItem> + Send>> {
        if let Some(event) = event {
            let is_generator_file = ProjectGenerator::is_genertor_file(event.path());
            if (event.is_content_update_event() && is_generator_file)
                || (event.is_create_event() || event.is_remove_event())
            {
                self.generator.regenerate(&self.root).await
            } else {
                Ok(None)
            }
        } else {
            self.generator.regenerate(&self.root).await
        }
    }
}

use crate::util::fs::{
    get_build_cache_dir, get_build_cache_dir_with_config, gitignore_to_glob_patterns,
};

impl Project {
    /// Generate compile commands for project via compiling all targets
    pub async fn generate_compile_commands(&self) -> Result<()> {
        use xclog::{XCCompilationDatabase, XCCompileCommand};

        log::info!("Generating compile commands ... ");
        let mut compile_commands: Vec<XCCompileCommand> = vec![];
        let cache_root = get_build_cache_dir(&self.root)?;
        // Because xcodebuild clean can't remove it
        tokio::fs::remove_dir_all(&cache_root).await.ok();

        let build_args: Vec<String> = vec![
            "clean".into(),
            "build".into(),
            format!("SYMROOT={cache_root}"),
            "-configuration".into(),
            "Debug".into(),
            "CODE_SIGN_IDENTITY=\"\"".into(),
            "CODE_SIGNING_REQUIRED=\"NO\"".into(),
            "CODE_SIGN_ENTITLEMENTS=\"\"".into(),
            "CODE_SIGNING_ALLOWED=\"NO\"".into(),
        ];
        println!("{build_args:#?}");
        let compile_db = XCCompilationDatabase::generate(&self.root, &build_args).await?;

        compile_db
            .into_iter()
            .for_each(|cmd| compile_commands.push(cmd));

        log::info!("Compile Commands Generated");

        let json = serde_json::to_vec_pretty(&compile_commands)?;
        tokio::fs::write(self.root.join(".compile"), &json).await?;

        Ok(())
    }

    pub fn build_args(
        &self,
        build_settings: &BuildSettings,
        device: &Option<Device>,
    ) -> Result<Vec<String>> {
        let mut args = build_settings
            .to_string()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        args.remove(0);
        args.insert(0, "build".to_string());

        if let Some(device) = device {
            args.extend(device.special_build_args())
        }

        // TODO: check if scheme isn't already defined!
        args.extend_from_slice(&[
            format!(
                "SYMROOT={}",
                get_build_cache_dir_with_config(&self.root, build_settings)?
            ),
            "-allowProvisioningUpdates".into(),
        ]);

        Ok(args)
    }
}
