use crate::notification::Timeout;
use serde::Deserialize;
use std::num::NonZeroU8;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    /// Notification on startup
    /// pub greet: bool,

    /// The number of simultaneous notifications displayed on the screen
    pub max_active: NonZeroU8,

    /// The number of
    pub max_history: usize,

    /// Space between windows
    pub spacing: u16,

    pub window: WindowRules,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            spacing: 10,
            max_active: NonZeroU8::new(5).unwrap(),
            max_history: 64,
            window: Default::default(),
        }
    }
}

/// Setting for notification window
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WindowRules {
    pub content: ContentRules,

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
            content: Default::default(),
            hgap: 10,
            vgap: 10,
            expire_timeout: Default::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentRules {
    /// Icon size
    pub icon_size: u16,

    /// Letters
    pub max_body: u16,
}

impl Default for ContentRules {
    fn default() -> Self {
        Self {
            icon_size: 32,
            max_body: 60,
        }
    }
}
