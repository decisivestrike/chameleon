use crate::providers::applications::ApplicationProvider;
use crate::providers::{Provider, WallpapersProvider};
use serde::Deserialize;
use std::collections::HashSet;
use std::env::home_dir;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationProviderConfig {
    pub terminal_cmd: Option<String>,
    pub detach: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WallpapersProviderConfig {
    #[serde(default = "default_wallpapers_path")]
    pub path: PathBuf,

    #[serde(default = "default_change_command")]
    pub change_command: String,
}

fn default_wallpapers_path() -> PathBuf {
    home_dir().unwrap().join("/Pictures")
}

fn default_change_command() -> String {
    "awww img {{image}}".to_string()
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LauncherConfig {
    pub searchbar_placeholder: String,
    pub providers: ProviderList,

    #[serde(rename = "applications")]
    pub applications_provider_config: ApplicationProviderConfig,

    #[serde(rename = "wallpapers")]
    pub wallpapers_provider_config: WallpapersProviderConfig,
}

impl LauncherConfig {
    pub fn instantiate_providers(self) -> Vec<Rc<dyn Provider>> {
        let Self {
            providers,
            applications_provider_config,
            wallpapers_provider_config,
            ..
        } = self;

        let mut provider_instances = Vec::with_capacity(providers.0.len());

        for p in providers.0.iter() {
            let instance: Rc<dyn Provider> = match p {
                ProviderVariant::Applications => {
                    Rc::new(ApplicationProvider::new(
                        applications_provider_config.clone(),
                    ))
                }
                ProviderVariant::Wallpapers => Rc::new(
                    WallpapersProvider::new(wallpapers_provider_config.clone()),
                ),
            };

            provider_instances.push(instance);
        }

        provider_instances
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ProviderList(HashSet<ProviderVariant>);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum ProviderVariant {
    Applications,
    Wallpapers,
}
