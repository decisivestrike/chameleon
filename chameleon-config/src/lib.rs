pub mod bar;
pub use bar::Bar;

pub mod widgets;
pub use widgets::Widgets;

use anyhow::{Result, bail};
use log::error;
use serde::Deserialize;
use std::{
    fmt,
    path::{Path, PathBuf},
    rc::Rc,
    sync::OnceLock,
};

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: Rc<Widgets>,
    #[serde(default, rename = "taskbar")] // Statusbar panel
    pub bar: Rc<Bar>,
}

impl Config {
    pub fn init<P>(path: P) -> Self
    where
        P: AsRef<Path> + fmt::Debug,
    {
        CONFIG_PATH.get_or_init(|| path.as_ref().to_path_buf());

        match Self::update() {
            Ok(config) => config,
            Err(e) => {
                error!("{e}");
                std::process::exit(-1);
            }
        }
    }

    pub fn update() -> Result<Self> {
        let path = CONFIG_PATH.get().unwrap();

        let toml_str = match std::fs::read_to_string(&path) {
            Ok(file) => file,
            Err(e) => {
                bail!("Failed to open config file at {path:?}. {e}")
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => Ok(config),
            Err(e) => bail!("{e}"),
        }
    }
}
