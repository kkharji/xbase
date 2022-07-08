//! Functions to query/access filesystem
use crate::BuildSettings;
use anyhow::Result;
use std::{fmt::Debug, path::Path};
use tap::Pipe;
use tokio::fs;

pub fn get_dirname_dir_root(path: impl AsRef<Path>) -> Option<String> {
    let path = path.as_ref();
    path.strip_prefix(path.ancestors().nth(2)?)
        .ok()?
        .display()
        .to_string()
        .replace("/", "_")
        .pipe(Some)
}

/// Ensure single socket server and process running
pub async fn ensure_one_instance(pid_path: &'static str, sock_addr: &'static str) -> Result<()> {
    if fs::metadata(sock_addr).await.ok().is_some() {
        fs::remove_file(sock_addr).await.ok();
        if fs::metadata(pid_path).await.ok().is_some() {
            fs::read_to_string(pid_path)
                .await?
                .pipe_ref(super::pid::kill)
                .await?;
        }
        fs::remove_file(pid_path).await.ok();
    }

    fs::write(pid_path, std::process::id().to_string()).await?;

    Ok(())
}

pub fn _get_build_cache_dir<P: AsRef<Path> + Debug>(
    root_path: P,
    config: Option<&BuildSettings>,
) -> Result<String> {
    let path = || {
        let name = get_dirname_dir_root(&root_path)?;
        let base = dirs::cache_dir()?
            .join("Xbase")
            .join(name)
            .display()
            .to_string();

        if let Some(config) = config {
            let target = &config.target;
            let config = config.configuration.to_string();
            Some(format!("{base}/{target}_{config}",).replace(" ", "_"))
        } else {
            Some(base)
        }
    };
    path()
        .ok_or_else(|| anyhow::anyhow!("Fail to generate build_cache directory for {root_path:?}"))
}

pub fn get_build_cache_dir<P: AsRef<Path> + Debug>(root_path: P) -> Result<String> {
    _get_build_cache_dir(root_path, None)
}

pub fn get_build_cache_dir_with_config<P: AsRef<Path> + Debug>(
    root_path: P,
    config: &BuildSettings,
) -> Result<String> {
    _get_build_cache_dir(root_path, Some(config))
}

/// Get path to binary by name
pub fn which(cmd: &str) -> Result<String> {
    Ok(which::which(cmd)?.to_str().unwrap().to_string())
}

/// Read .gitignore from root and return vec of glob patterns if the .gitignore eixists.
pub async fn gitignore_to_glob_patterns<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let gitignore_path = path.as_ref().join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(Default::default());
    }
    let content = tokio::fs::read_to_string(gitignore_path).await?;
    Ok(gitignore_content_to_glob_patterns(content))
}

pub fn gitignore_content_to_glob_patterns(content: String) -> Vec<String> {
    content
        .split("\n")
        .filter(|s| !s.is_empty() && !s.starts_with("#"))
        .flat_map(|s| {
            if s.starts_with("!") {
                None // TODO(watchignore): support ! patterns
            } else {
                Some(("", s))
            }
        })
        .filter(|&(_, s)| s.chars().next() != Some('/'))
        .map(|(pat, s)| {
            if s != "/" && !s.starts_with("**") {
                (pat, format!("**/{s}"))
            } else {
                (pat, format!("{s}"))
            }
        })
        .flat_map(|(pat, s)| {
            let pattern = format!("{pat}{s}");
            vec![
                pattern.clone(),
                if pattern.ends_with("/") {
                    format!("{pattern}**")
                } else {
                    format!("{pattern}/**")
                },
            ]
        })
        .collect::<Vec<String>>()
}

#[test]
fn test_gitignore_patterns() {
    let gitignore_patterns =
        gitignore_content_to_glob_patterns(String::from(".build.log\n.compile"));
    assert_eq!(
        gitignore_patterns,
        vec![
            "**/.build.log".to_string(),
            "**/.build.log/**".to_string(),
            "**/.compile".to_string(),
            "**/.compile/**".to_string()
        ]
    );

    println!("{gitignore_patterns:#?}");
}
