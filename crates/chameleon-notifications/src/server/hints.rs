use crate::server::image_data::ImageData;
use zbus::zvariant::{DeserializeDict, Type};

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "a{sv}")]
pub struct NotificationHints {
    pub action_icons: Option<bool>,
    pub category: Option<String>,
    pub desktop_entry: Option<String>,
    pub image_data: Option<ImageData>,
    pub image_path: Option<String>,
    pub resident: Option<bool>,
    pub sound_file: Option<String>,
    pub sound_name: Option<String>,
    pub suppress_sound: Option<bool>,
    pub transient: Option<bool>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub urgency: Option<u8>,
}
