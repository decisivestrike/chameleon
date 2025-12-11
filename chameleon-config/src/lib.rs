pub mod bar;
pub use bar::Bar;

pub mod widgets;
pub use widgets::Widgets;

use anyhow::{Result, bail};
use grapes::tokio::sync::broadcast::{self, Receiver, Sender};
use log::error;
use serde::Deserialize;
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
};

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static TX: LazyLock<Sender<Arc<Config>>> =
    LazyLock::new(|| broadcast::channel(64).0);

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: Widgets,
    #[serde(default, rename = "taskbar")] // Statusbar panel
    pub bar: Bar,
}

impl Config {
    pub fn init<P>(path: P)
    where
        P: AsRef<Path> + fmt::Debug,
    {
        CONFIG_PATH.get_or_init(|| path.as_ref().to_path_buf());

        if let Err(e) = Self::update() {
            error!("{e}");
            std::process::exit(-1);
        }
    }

    pub fn subscribe() -> Receiver<Arc<Config>> {
        TX.subscribe()
    }

    fn update() -> Result<()> {
        let path = CONFIG_PATH.get().unwrap();

        let toml_str = match std::fs::read_to_string(&path) {
            Ok(file) => file,
            Err(e) => {
                bail!("Failed to open config file at {path:?}. {e}")
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => {
                TX.send(Arc::new(config)).unwrap();
                Ok(())
            }

            Err(e) => bail!("{e}"),
        }
    }
}
