use crate::types::BuildConfiguration;
use crate::util::fs::get_build_cache_dir_with_config;
use crate::Result;
use crate::{nvim::Logger, util::fs::get_build_cache_dir};
use async_stream::stream;
use process_stream::{Stream, StreamExt};
use std::path::Path;
use tap::Pipe;
use xcodebuild::{parser, runner::spawn};

#[cfg(feature = "daemon")]
use std::fmt::Debug;

#[cfg(feature = "daemon")]
pub async fn stream_build<'a, P: 'a>(
    root: P,
    args: &'a Vec<String>,
) -> Result<impl Stream<Item = String> + 'a>
where
    P: AsRef<Path>,
{
    let mut stream = spawn(root, args).await?;

    Ok(Box::pin(stream! {
        use xcodebuild::parser::Step::*;
        while let Some(step) = stream.next().await {
            let line = match step {
                Exit(v) if v != 0 => {
                    "[Error] Build Failed".into()
                }
                BuildSucceed | CleanSucceed | TestSucceed | TestFailed | BuildFailed => {
                    continue;
                }
                step => step.to_string().trim().to_string(),
            };
            if !line.is_empty() {
                for line in line.split("\n") {
                    yield line.to_string();
                }
            }
        }
    }))
}

#[cfg(feature = "daemon")]
pub async fn fresh_build<'a, P: AsRef<Path> + 'a + Debug>(
    root: P,
) -> Result<impl Stream<Item = parser::Step> + 'a> {
    append_build_root(&root, None, vec!["clean".into(), "build".into()])?
        .pipe(|args| spawn(root, args))
        .await?
        .pipe(Ok)
}

pub async fn build_with_logger<'a, P: AsRef<Path>>(
    logger: &mut Logger<'a>,
    root: P,
    args: &Vec<String>,
    clear: bool,
    open: bool,
) -> Result<bool> {
    let mut stream = crate::xcode::stream_build(root, args).await?;

    // TODO(nvim): close log buffer if it is open for new direction
    //
    // Currently the buffer direction will be ignored if the buffer is opened already

    if clear {
        logger.clear_content().await?;
    }

    // TODO(nvim): build log correct height
    if open {
        logger.open_win().await?;
    }

    let mut success = true;

    logger.set_running().await?;

    while let Some(line) = stream.next().await {
        line.contains("FAILED").then(|| success = false);

        logger.append(line).await?;
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
        get_build_cache_dir_with_config(&root, config)?
    } else {
        get_build_cache_dir(&root)?
    }
    .pipe(|path| format!("SYMROOT={path}|CONFIGURATION_BUILD_DIR={path}|BUILD_DIR={path}"))
    .split("|")
    .map(ToString::to_string)
    .collect::<Vec<String>>()
    .pipe(|extra| {
        args.extend(extra);
        args
    })
    .pipe(Ok)
}
