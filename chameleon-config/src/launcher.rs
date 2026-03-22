use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub enabled: bool,
    pub placeholder: String,

    /// Command for launching terminal apps
    pub terminal_cmd: Option<String>,

    /// Child process detach using libc setpid
    pub detach: bool,
}
