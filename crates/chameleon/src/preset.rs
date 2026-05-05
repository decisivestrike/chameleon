use chameleon_shared::utils::read_config;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Preset {
    /// Enabled modules
    pub enabled: HashSet<Module>,

    /// Path to folder with themes
    pub themes_root: Option<PathBuf>,

    /// Default theme for preset
    pub theme: String,
}

impl Preset {
    pub fn load(path: impl AsRef<Path>) -> Self {
        read_config(&path).unwrap()
    }
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Launcher,
    Notifications,
    Panel,
    Watcher,
    Widgets,
}

impl AsRef<str> for Module {
    fn as_ref(&self) -> &str {
        match self {
            Module::Launcher => "chameleon-launcher",
            Module::Notifications => "chameleon-notifications",
            Module::Panel => "chameleon-panel",
            Module::Watcher => "chameleon-watcher",
            Module::Widgets => "chameleon-widgets",
        }
    }
}
