use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Battery {
    pub icons: Vec<String>,
    pub name: String,
    pub format: String,
}
