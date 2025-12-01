pub mod bar;
pub use bar::Bar;

pub mod widgets;
pub use widgets::Widgets;

use log::error;
use serde::Deserialize;
use std::sync::LazyLock;

const CONFIG_PATH: &str = "chameleon.toml"; // remove it

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::read());

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: Widgets,
    #[serde(default, rename = "taskbar")]
    pub bar: Bar,
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
