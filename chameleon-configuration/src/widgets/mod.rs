use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Widgets {
    #[serde(default)]
    pub enabled: bool,
}
