#![allow(unused)]

mod runtime;

use safer_ffi::prelude::*;
use std::path::PathBuf;
use xbase_proto::*;

#[ffi_export]
fn xbase_hello() -> char_p::Box {
    String::from("Hello, world!").try_into().unwrap()
}

// parse string to a vector of messages
fn parse(content: String) -> Result<Vec<Message>> {
    fn parse_single(line: &str) -> Result<Message> {
        serde_json::from_str(&line)
            .map_err(|e| Error::MessageParse(format!("\n{line}\n Error: {e}")))
    }
    let mut vec = vec![];
    if content.contains("\n") {
        for message in content
            .split("\n")
            .filter(|s| !s.trim().is_empty())
            .map(parse_single)
        {
            vec.push(message?);
        }
    } else {
        vec.push(parse_single(&content)?);
    }
    Ok(vec)
}
