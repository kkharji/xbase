//! General utilities

pub mod fmt;
pub mod fs;
pub mod pid;

use crate::OutputStream;
use process_stream::{ProcessItem, StreamExt};
use xbase_proto::Client;

/// Consume given stream and return whether the stream exist with 0
/// TODO(project): log build and compile logs to client
pub async fn consume_and_log(mut stream: OutputStream) -> (bool, Vec<String>) {
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

pub fn handler_log_content(reqname: &str, client: &Client) -> (String, String) {
    let title = format!(
        "{reqname} [{}:{}]..................",
        client.abbrev_root(),
        client.pid,
    );
    let sep = ".".repeat(title.len());
    (title, sep)
}
