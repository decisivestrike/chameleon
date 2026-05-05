use crate::notification::Timeout;
use serde::Deserialize;
use std::num::NonZeroU8;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    /// Notification on startup
    // pub greet: bool,

    /// The number of simultaneous notifications displayed on the screen
    pub max_active: NonZeroU8,

    /// The number of
    pub max_history: usize,

    /// Gap between windows
    pub gap: u16,

    #[serde(rename = "window")]
    pub window_rules: WindowRules,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            gap: 10,
            max_active: NonZeroU8::new(5).unwrap(),
            max_history: 64,
            window_rules: Default::default(),
        }
    }
}

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
