use serde::Deserialize;
use std::collections::HashSet;

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub apps: HashSet<String>,

    #[serde(default)]
    pub terminal_cmd: Option<String>,

    #[serde(default)]
    pub detach: bool,
}
