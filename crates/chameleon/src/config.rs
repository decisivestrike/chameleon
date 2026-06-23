use chameleon_shared::utils::{read_config, resolve_path};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::process::exit;
use tracing::error;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub modules: HashSet<Module>,

    /// Default theme for preset
    pub theme: Option<String>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Self {
        match read_config(&path) {
            Ok(config) => config,
            Err(e) => {
                error!("{}", e);
                exit(1)
            }
        }
    }

    pub fn args(&self) -> Vec<String> {
        if let Some(theme) = &self.theme {
            let theme_path = resolve_path(&format!(
                "~/.config/chameleon/themes/{theme}.css"
            ))
            .unwrap()
            .to_string_lossy()
            .to_string();

            vec!["-s".to_string(), theme_path]
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Launcher,
    Notifications,
    Panel,
    Watcher,
    Widgets,
    Dock,
}

impl AsRef<str> for Module {
    fn as_ref(&self) -> &str {
        match self {
            Module::Launcher => "chameleon-launcher",
            Module::Notifications => "chameleon-notifications",
            Module::Panel => "chameleon-panel",
            Module::Watcher => "chameleon-watcher",
            Module::Widgets => "chameleon-widgets",
            Module::Dock => "chameleon-dock",
        }
    }
}
