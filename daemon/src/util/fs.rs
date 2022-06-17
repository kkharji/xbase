//! Functions to query filesystem for files and directories
use anyhow::Result;
use std::path::Path;
use tap::Pipe;
use xbase_proto::BuildSettings;

pub fn get_dirname_dir_root<P>(path: P) -> Option<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    path.strip_prefix(path.ancestors().nth(2)?)
        .ok()?
        .display()
        .to_string()
        .replace("/", "_")
        .pipe(Some)
}

pub fn _get_build_cache_dir<P: AsRef<Path> + std::fmt::Debug>(
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
            Some(
                format!(
                    "{base}/{}_{}",
                    config.target,
                    config.configuration.to_string()
                )
                .replace(" ", "_"),
            )
        } else {
            Some(base)
        }
    };
    path()
        .ok_or_else(|| anyhow::anyhow!("Fail to generate build_cache directory for {root_path:?}"))
}

pub fn get_build_cache_dir<P: AsRef<Path> + std::fmt::Debug>(root_path: P) -> Result<String> {
    _get_build_cache_dir(root_path, None)
}

pub fn get_build_cache_dir_with_config<P: AsRef<Path> + std::fmt::Debug>(
    root_path: P,
    config: &BuildSettings,
) -> Result<String> {
    _get_build_cache_dir(root_path, Some(config))
}
