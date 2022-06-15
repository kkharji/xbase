use crate::nvim::Logger;
use crate::types::BuildConfiguration;
use crate::Result;
use process_stream::StreamExt;
use std::path::Path;
use tap::Pipe;
use xclog::XCLogger;

#[cfg(feature = "daemon")]
use std::fmt::Debug;

pub async fn build_with_logger<'a, P: AsRef<Path>>(
    logger: &mut Logger<'a>,
    root: P,
    args: &Vec<String>,
    clear: bool,
    open: bool,
) -> Result<bool> {
    let mut success = true;
    let mut xclogger = XCLogger::new(root.as_ref(), args)?;

    // TODO(nvim): close log buffer if it is open for new direction
    // Currently the buffer direction will be ignored if the buffer is opened already

    if clear {
        logger.clear_content().await?;
    }

    // TODO(nvim): build log correct height
    if open {
        logger.open_win().await?;
    }

    logger.set_running(false).await?;

    while let Some(line) = xclogger.next().await {
        line.contains("FAILED").then(|| success = false);

        logger.append(line.to_string()).await?;
    }

    logger.set_status_end(success, open).await?;

    Ok(success)
}

pub fn append_build_root<P: AsRef<Path> + Debug>(
    root: P,
    config: Option<&BuildConfiguration>,
    mut args: Vec<String>,
) -> Result<Vec<String>> {
    if let Some(config) = config {
        crate::util::fs::get_build_cache_dir_with_config(&root, config)?
    } else {
        crate::util::fs::get_build_cache_dir(&root)?
    }
    .pipe(|path| format!("SYMROOT={path}|CONFIGURATION_BUILD_DIR={path}"))
    .split("|")
    .map(ToString::to_string)
    .collect::<Vec<String>>()
    .pipe(|extra| {
        args.extend(extra);
        args.push("-allowProvisioningUpdates".to_string());
        args
    })
    .pipe(Ok)
}
