pub mod bar;
pub use bar::Bar;

pub mod widgets;
use grapes::tokio::sync::RwLock;
pub use widgets::Widgets;

use log::error;
use serde::Deserialize;
use std::{
    fmt,
    path::{Path, PathBuf},
};

pub static CONFIG: RwLock<Option<Config>> = RwLock::const_new(None);

#[derive(Debug, Deserialize)]
pub struct ConfigInner {
    #[serde(default, rename = "widgets")]
    pub widgets: Widgets,
    #[serde(default, rename = "taskbar")]
    pub bar: Bar,
}

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    inner: ConfigInner,
}

impl Config {
    fn new(path: impl AsRef<Path>, inner: ConfigInner) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            inner,
        }
    }

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
            Ok(inner) => {
                *CONFIG.blocking_write() = Some(Config::new(path, inner))
            }
            Err(e) => {
                error!("{e}");
                std::process::exit(-1);
            }
        };
    }

    fn update(&self) {
        let toml_str = match std::fs::read_to_string(&self.path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open config file at {:?}. {e}", self.path);
                return;
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => *CONFIG.blocking_write() = config,
            Err(e) => {
                error!("{e}");
                std::process::exit(-1);
            }
        };

        // broadcast
    }

    pub fn read(config: &'static RwLock<Option<Self>>) -> &'static ConfigInner {
        &config.inner
    }
}
