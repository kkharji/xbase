//! General utilities

pub mod fmt;
pub mod fs;
pub mod pid;

use crate::OutputStream;
use process_stream::{ProcessItem, StreamExt};

/// Consume given stream and return whether the stream exist with 0
/// TODO(project): log build and compile logs to client
pub async fn consume_and_log(mut stream: OutputStream) -> (bool, Vec<String>) {
    let mut success = false;
    let mut items = vec![];
    while let Some(output) = stream.next().await {
        if let ProcessItem::Exit(v) = output {
            success = v.eq("0");
        } else {
            log::info!("{output}");
            items.push(output.to_string())
        }
    }
    (success, items)
}
