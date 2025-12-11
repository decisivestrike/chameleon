pub mod bar;
pub use bar::Bar;

pub mod widgets;
pub use widgets::Widgets;

use log::error;
use serde::Deserialize;
use std::{fmt, path::Path, sync::OnceLock};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: Widgets,
    #[serde(default, rename = "taskbar")]
    pub bar: Bar,
}

impl Config {
    pub fn init<P>(path: P)
    where
        P: AsRef<Path> + fmt::Debug,
    {
        let toml_str = match std::fs::read_to_string(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open config file at {path:?}. {e}");
                std::process::exit(-1);
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => CONFIG.get_or_init(|| config),
            Err(e) => {
                error!("{e}");
                std::process::exit(-1);
            }
        };
    }

    pub fn read() -> &'static Self {
        CONFIG.get_or_init(|| unreachable!())
    }
}
