use crate::{bar::BarConfig, widgets::WidgetsConfig};
use log::error;
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
        let toml_str = match std::fs::read_to_string("chameleon.toml") {
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

// #[derive(Debug, Deserialize)]
// pub struct Battery {
//     pub icons: Vec<String>,
//     pub name: String,
//     pub format: String,
// }
