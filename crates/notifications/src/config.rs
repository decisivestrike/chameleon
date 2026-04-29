use crate::requests::notification_data::Timeout;
use serde::Deserialize;
use std::num::NonZeroU8;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationsConfig {
    pub icon_size: u16,
    pub expire_timeout: Timeout,

    /// The number of simultaneous notifications displayed on the screen
    pub max_notifications: NonZeroU8,

    /// Letters
    pub max_body: u16,

    /// Gaps between windows
    pub gaps: u16,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            icon_size: 32,
            expire_timeout: Default::default(),
            max_notifications: NonZeroU8::new(5).unwrap(),
            max_body: 80,
            gaps: 10,
        }
    }
}
