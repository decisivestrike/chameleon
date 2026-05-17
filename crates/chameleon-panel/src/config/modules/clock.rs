use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ClockRules {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "%H:%M".to_string()
}
