use crate::cli::Args;
use crate::notification::Timeout;
use arc_swap::ArcSwap;
use chameleon_shared::utils::read_config;
use serde::Deserialize;
use std::num::NonZeroU8;
use std::sync::LazyLock;

pub static RULES: LazyLock<ArcSwap<Rules>> = LazyLock::new(|| {
    let args: Args = argh::from_env();
    let rules: Rules = read_config(args.config_path).unwrap();

    ArcSwap::from_pointee(rules)
});

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    #[serde(flatten)]
    pub common_rules: CommonRules,

    pub window: WindowRules,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CommonRules {
    /// Notification on startup
    /// pub greet: bool,

    /// The number of simultaneous notifications displayed on the screen
    pub max_active: NonZeroU8,

    /// The number of
    pub max_history: usize,

    /// Space between windows
    pub spacing: u16,
}

impl Default for CommonRules {
    fn default() -> Self {
        Self {
            spacing: 10,
            max_active: NonZeroU8::new(5).unwrap(),
            max_history: 64,
        }
    }
}

/// Setting for notification window
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WindowRules {
    /// Icon size
    pub icon_size: u16,

    /// Letters
    pub max_body: u16,

    /// Horizontal gap
    pub hgap: u16,

    /// Vertical gap
    pub vgap: u16,

    /// Default notification timeout
    pub expire_timeout: Timeout,
}

impl Default for WindowRules {
    fn default() -> Self {
        Self {
            icon_size: 32,
            max_body: 60,
            hgap: 10,
            vgap: 10,
            expire_timeout: Default::default(),
        }
    }
}
