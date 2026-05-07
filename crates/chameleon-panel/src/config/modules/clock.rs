use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ClockConfig {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "%H:%M".to_string()
}
