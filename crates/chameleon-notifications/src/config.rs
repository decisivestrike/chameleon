use crate::requests::notification_data::Timeout;
use serde::{Deserialize, Deserializer};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationsConfig {
    #[serde(default = "default_icon_size")]
    pub icon_size: u16,
    pub expire_timeout: Timeout,

    /// The number of simultaneous notifications displayed on the screen
    #[serde(
        default = "default_max_notifications_count",
        deserialize_with = "deserialize_max_notifications"
    )]
    pub max_notifications: u8,

    /// Letters
    #[serde(default = "default_max_body_len")]
    pub max_body: u16,

    /// Gaps between windows
    #[serde(default = "default_gap")]
    pub gaps: u16,
}

fn deserialize_max_notifications<'de, D>(
    deserializer: D,
) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    if value == 0 {
        return Err(serde::de::Error::custom("max notifications must be >= 1"));
    }

    Ok(value)
}

fn default_icon_size() -> u16 {
    32
}

fn default_max_notifications_count() -> u8 {
    5
}

fn default_max_body_len() -> u16 {
    80
}

fn default_gap() -> u16 {
    10
}
