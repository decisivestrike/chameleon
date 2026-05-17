use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct BatteryRules {
    pub icons: Vec<String>,
    pub name: String,
    pub format: String,
}
