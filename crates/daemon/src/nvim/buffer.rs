use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BufferDirection {
    Default,
    Vertical,
    Horizontal,
    TabEdit,
}

impl Default for BufferDirection {
    fn default() -> Self {
        Self::Default
    }
}

impl BufferDirection {
    pub fn to_nvim_command(&self, bufnr: i64) -> String {
        match self {
            // TOOD: support build log float as default
            BufferDirection::Default => format!("sbuffer {bufnr}"),
            BufferDirection::Vertical => format!("vert sbuffer {bufnr}"),
            BufferDirection::Horizontal => format!("sbuffer {bufnr}"),
            BufferDirection::TabEdit => format!("tabe {bufnr}"),
        }
    }
}
