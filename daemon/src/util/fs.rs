//! Functions to query filesystem for files and directories
use anyhow::Result;
use std::{fmt::Debug, path::Path};
use tap::Pipe;
use xbase_proto::BuildSettings;

pub trait PathExt {
    fn name(&self) -> Option<String>;
    fn unique_name(&self) -> Option<String>;
    fn abbrv(&self) -> Result<&Path>;
}

impl PathExt for Path {
    fn name(&self) -> Option<String> {
        self.file_name()
            .and_then(|os| os.to_str())
            .map(ToString::to_string)
    }

    fn unique_name(&self) -> Option<String> {
        self.strip_prefix(self.ancestors().nth(3)?)
            .ok()?
            .display()
            .to_string()
            .replace("/", "_")
            .pipe(Some)
    }

    fn abbrv(&self) -> Result<&Path> {
        let ancestors = self.ancestors().nth(3);
        let ancestors = ancestors
            .ok_or_else(|| crate::Error::Unexpected("Getting 3 parent of a path".into()))?;
        Ok(self.strip_prefix(ancestors)?)
    }
}

pub fn get_dirname_dir_root(path: impl AsRef<Path>) -> Option<String> {
    let path = path.as_ref();
    path.strip_prefix(path.ancestors().nth(2)?)
        .ok()?
        .display()
        .to_string()
        .replace("/", "_")
        .pipe(Some)
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
pub fn abbrv_path<P: AsRef<Path>>(path: P) -> String {
    let abbr = || {
        let path = path.as_ref();

        Some(
            path.strip_prefix(path.ancestors().nth(3)?)
                .ok()?
                .display()
                .to_string(),
        )
    };

    abbr().unwrap_or_default()
}
