use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub enum ProjectGenerator {
    /// No Generator
    None,
    /// XCodeGen Generator
    XCodeGen,
    /// Tuist Generator
    Tuist,
}

impl Default for ProjectGenerator {
    fn default() -> Self {
        Self::None
    }
}

impl ProjectGenerator {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref();
        if root.join("project.yml").exists() {
            Self::XCodeGen
        } else if root.join("Project.swift").exists() {
            Self::Tuist
        } else {
            Self::None
        }
    }
}
