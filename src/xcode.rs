use anyhow::Result;
use async_stream::stream;
use std::path::Path;
use tap::Pipe;
use tokio_stream::{Stream, StreamExt};
use xcodebuild::{parser, runner::spawn};

use crate::util::fs::get_build_cache_dir;

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
                Exit(v) => {
                    crate::util::string_as_section(if v == "0" { "Succeed" } else { "Failed" }.to_string())
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
pub async fn fresh_build<'a, P: AsRef<Path> + 'a + std::fmt::Debug>(
    root: P,
) -> Result<impl Stream<Item = parser::Step> + 'a> {
    /*
       TODO: Support overriding xcodebuild arguments

       Not sure how important is this, but ideally I'd like to be able to add extra arguments for
       when generating compiled commands, as well as doing actual builds and runs.

       ```yaml
       xbase:
       buildArguments: [];
       compileArguments: [];
       runArguments: [];
       ```
    */
    append_build_root(&root, vec!["clean".into(), "build".into()])?
        .pipe(|args| spawn(root, args))
        .await
}

pub fn append_build_root<P: AsRef<Path> + std::fmt::Debug>(
    root: P,
    mut args: Vec<String>,
) -> Result<Vec<String>> {
    get_build_cache_dir(&root)?
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
