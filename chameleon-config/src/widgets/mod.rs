use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WidgetsConfig {
    #[serde(default)]
    pub enabled: bool,
}
