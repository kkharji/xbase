use anyhow::{anyhow, Result};
use bsp_server::types::Url;
use serde_json::Value;
use std::path::Path;
use tap::Pipe;

use std::{fs::read_to_string, path::PathBuf};

/// Try to get indexStorePath from config_filepath or default to "{cache_path}/indexStorePath"
pub fn get_index_store_path(cache_path: &String, config_filepath: &PathBuf) -> String {
    let mut index_store_path = format!("{cache_path}/indexStorePath");
    if config_filepath.is_file() {
        if let Ok(content) = read_to_string(config_filepath) {
            if let Ok(config) = serde_json::from_str::<Value>(&content) {
                if let Some(Value::String(p)) = config.get("indexStorePath") {
                    index_store_path = p.clone()
                }
            };
        };
    }
    index_store_path
}

/// Try to get .compile filepath
pub fn get_compile_filepath(url: &Url) -> Option<PathBuf> {
    url.join(".compile")
        .ok()?
        .path()
        .pipe(PathBuf::from)
        .pipe(|path| path.is_file().then(|| path))
}

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

pub fn get_build_cache_dir<P>(root_path: P) -> Result<String>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let path = || {
        let name = get_dirname_dir_root(&root_path)?;
        let base = dirs::cache_dir()?
            .join("Xbase")
            .join(name)
            .display()
            .to_string();
        Some(base)
    };
    path().ok_or_else(|| anyhow!("Fail to generate build_cache directory for {root_path:?}"))
}
