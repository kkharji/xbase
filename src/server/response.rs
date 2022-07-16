use crate::{types::Result, ServerError};
use serde::Serialize;
use serde_json::Value;
use typescript_type_def::TypeDef;

/// Server Response
#[derive(Default, Debug, Serialize, TypeDef)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ServerError>,
}

impl Response {
    pub fn new<T: Serialize>(v: Result<T>) -> Response {
        let mut response = Response::default();
        match v {
            Ok(data) => response.data = serde_json::to_value(data).unwrap().into(),
            Err(ref error) => response.error = Some(error.into()),
        };
        response
    }
}
