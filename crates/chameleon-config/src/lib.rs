pub mod panel;
pub use panel::PanelConfig;

pub mod widgets;
pub use widgets::WidgetsConfig;

pub mod launcher;
pub use launcher::LauncherConfig;

use chameleon_cli::ARGS;
use serde::Deserialize;
use std::path::Path;
use std::sync::LazyLock;
use std::{fmt, fs};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config_path = &ARGS.config_path;
    Config::init(config_path)
});

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub widgets: WidgetsConfig,
    pub panel: PanelConfig,
    pub launcher: LauncherConfig,
}

impl Config {
    fn init<P>(config_path: P) -> Self
    where
        P: AsRef<Path> + fmt::Debug,
    {
        let toml_str = match fs::read_to_string(&config_path) {
            Ok(file) => file,
            Err(e) => {
                log::error!(
                    "Failed to open config file at {config_path:?}. {e}"
                );
                std::process::exit(-1);
            }
        };

        match toml::from_str(&toml_str) {
            Ok(config) => config,
            Err(e) => {
                log::error!("{e}");
                std::process::exit(-1);
            }
        }
    }
}
