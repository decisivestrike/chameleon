pub mod panel;
pub use panel::PanelConfig;

pub mod widgets;
pub use widgets::WidgetsConfig;

pub mod launcher;
pub use launcher::LauncherConfig;

use chameleon_cli::ARGS;
use serde::Deserialize;
use std::{fmt, fs, path::Path, sync::LazyLock};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config_path = &ARGS.config_path;
    Config::init(config_path)
});

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, rename = "widgets")]
    pub widgets: WidgetsConfig,
    #[serde(default, rename = "panel")]
    pub panel: PanelConfig,
    #[serde(default, rename = "launcher")]
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
