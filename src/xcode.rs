use crate::types::BuildConfiguration;
use anyhow::Result;
use async_stream::stream;
use std::path::Path;
use tokio_stream::{Stream, StreamExt};
use xcodebuild::{
    parser,
    runner::{spawn, spawn_once},
};

#[cfg(feature = "daemon")]
pub async fn stream_build<'a, P: 'a>(
    root: P,
    config: &BuildConfiguration,
) -> Result<impl Stream<Item = String> + 'a>
where
    P: AsRef<Path>,
{
    let mut stream = spawn(root, config.as_args()).await?;

    Ok(Box::pin(stream! {
        use xcodebuild::parser::Step::*;
        while let Some(step) = stream.next().await {
            let line = match step {
                Exit(v) => {
                    format! {
                        "[{}] ----------------------------------------------------",
                        if v == "0" { "Succeed" } else { "Failed" }
                    }
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
pub async fn fresh_build<'a, P: AsRef<Path> + 'a>(
    root: P,
) -> Result<impl Stream<Item = parser::Step> + 'a> {
    /*
       TODO: Find away to get commands ran without doing xcodebuild clean

       Right now, in order to produce compiled commands and for `xcodebuild build` to spit out ran
       commands, we need to first run xcodebuild clean.

       NOTE: This far from possilbe after some research
    */
    spawn_once(&root, &["clean"]).await?;

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

    spawn(root, &["build"]).await
}
