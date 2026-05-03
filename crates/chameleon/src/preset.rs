use chameleon_shared::config;
use chameleon_shared::errors::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Preset {
    /// Enabled modules
    enabled: Vec<Module>,
    ipc: bool,
    themes_root: PathBuf,

    /// Default theme for preset
    theme: String,
}

impl Preset {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        config::read(&path)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Launcher,
    Notifications,
    Panel,
    Watcher,
    Widgets,
}
