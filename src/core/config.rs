use crate::{bar::config::BarConfig, widgets::WidgetsConfig};
use log::error;
use serde::Deserialize;
use std::sync::LazyLock;

const CONFIG_PATH: &str = "chameleon.toml";

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::read());

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: WidgetsConfig,
    #[serde(default, rename = "taskbar")]
    pub bar: BarConfig,
}

impl Config {
    pub fn read() -> Self {
        let toml_str = match std::fs::read_to_string(CONFIG_PATH) {
            Ok(file) => file,
            Err(e) => {
                error!("Can't open config. {e}");
                std::process::exit(-1);
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => config,
            Err(e) => {
                error!("{e}");
                std::process::exit(-1);
            }
        }
    }
}
