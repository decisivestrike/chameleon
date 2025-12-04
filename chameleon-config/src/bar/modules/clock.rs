use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Clock {
    #[serde(default = "Clock::default_format")]
    pub format: String,
}

impl Clock {
    fn default_format() -> String {
        "%H:%M".to_string()
    }
}
