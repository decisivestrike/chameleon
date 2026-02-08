use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ClockConfig {
    #[serde(default = "ClockConfig::default_format")]
    pub format: String,
}

impl ClockConfig {
    fn default_format() -> String {
        "%H:%M".to_string()
    }
}
