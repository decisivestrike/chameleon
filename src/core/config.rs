use std::sync::LazyLock;

use serde::Deserialize;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::read());

#[derive(Debug, Deserialize)]
pub struct Config {
    pub widgets: Option<Widgets>,
    pub bar: Option<Bar>,
    pub battery: Option<Battery>,
    pub clock: Option<Clock>,
}

#[derive(Debug, Deserialize)]
pub struct Widgets {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Bar {
    #[serde(default)]
    pub enabled: bool,
    pub height: Option<i32>,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Battery {
    pub icons: Vec<String>,
    pub name: String,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct Clock {
    pub format: Option<String>,
}

impl Config {
    pub fn read() -> Self {
        let toml_str = std::fs::read_to_string("chameleon.toml").unwrap();
        let config: Config = toml::from_str(&toml_str).unwrap();

        config
    }
}
