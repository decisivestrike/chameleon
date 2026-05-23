use crate::providers::Provider;
use crate::providers::applications::ApplicationProvider;
use crate::providers::wallpapers::WallpapersProvider;
use serde::Deserialize;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationProviderConfig {
    pub terminal_cmd: Option<String>,
    pub detach: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LauncherConfig {
    pub searchbar_placeholder: String,
    pub providers: ProviderList,

    #[serde(rename = "applications")]
    pub applications_provider_config: ApplicationProviderConfig,
}

impl LauncherConfig {
    pub fn instantiate_providers(self) -> Vec<Rc<dyn Provider>> {
        let Self {
            providers,
            applications_provider_config,
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
                ProviderVariant::Wallpapers => {
                    Rc::new(WallpapersProvider::new())
                }
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
