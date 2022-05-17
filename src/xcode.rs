use anyhow::Result;
use async_stream::stream;
use std::path::Path;
use tokio_stream::{Stream, StreamExt};
use xcodebuild::{parser, runner::spawn};

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
                    crate::util::string_as_section(if v == "0" { "[Succeed]" } else { "[Failed]" }.to_string())
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
    spawn(root, &["clean", "build"]).await
}
