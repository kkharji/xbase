use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PluginConfig {
    pub ignore: Vec<String>,
}
