//! General utilities

pub mod fmt;
pub mod fs;
pub mod pid;

use process_stream::{ProcessItem, ProcessStream, StreamExt};
use std::path::PathBuf;
use xbase_proto::PathExt;

/// Consume given stream and return whether the stream exist with 0
/// TODO(project): log build and compile logs to client
pub async fn consume_and_log(mut stream: ProcessStream) -> (bool, Vec<String>) {
    let mut success = false;
    let mut items = vec![];
    while let Some(output) = stream.next().await {
        if let ProcessItem::Exit(v) = output {
            success = v.eq("0");
        } else {
            log::debug!("{output}");
            items.push(output.to_string())
        }
    }
    (success, items)
}

pub fn _log_content(reqname: &str, root: &PathBuf) -> (String, String) {
    let title = format!(
        "{reqname} [{}]..................",
        root.as_path().abbrv().unwrap().display(),
    );
    let sep = ".".repeat(title.len());
    (title, sep)
}

macro_rules! log_request {
    ($name:literal, $root:ident) => {{
        let (title, sep) = crate::util::_log_content($name, &$root);
        log::info!("{sep}",);
        log::info!("{title}",);
        log::info!("{sep}",);
        sep
    }};
    ($name:literal, $root:ident, $req:ident) => {{
        let (title, sep) = crate::util::_log_content($name, &$root);
        log::info!("{sep}");
        log::info!("{title}");
        log::trace!("\n\n{:#?}\n", $req);
        log::info!("{sep}");
        sep
    }};
}

pub(crate) use log_request;
