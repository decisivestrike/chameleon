use crate::{bar::BarConfig, widgets::WidgetsConfig};
use serde::Deserialize;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::read());

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: WidgetsConfig,
    #[serde(default, rename = "bar")]
    pub bar: BarConfig,
}

impl Config {
    pub fn read() -> Self {
        let toml_str = std::fs::read_to_string("chameleon.toml").unwrap();
        let config: Config = toml::from_str(&toml_str).unwrap();

        config
    }
}

// #[derive(Debug, Deserialize)]
// pub struct Battery {
//     pub icons: Vec<String>,
//     pub name: String,
//     pub format: String,
// }
