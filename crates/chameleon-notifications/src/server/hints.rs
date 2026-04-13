use gtk::{
    gdk::{self, MemoryTexture},
    glib,
};
use serde::Deserialize;
use std::{collections::HashMap, fmt};
use zbus::zvariant::{DeserializeDict, OwnedValue, Type};

#[derive(Debug, DeserializeDict)]
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

#[derive(Type, Deserialize, OwnedValue)]
#[zvariant(signature = "(iiibiiay)")]
pub struct ImageData {
    pub width: i32,
    pub height: i32,
    pub rowstride: i32,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    pub data: Vec<u8>,
}

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("rowstride", &self.rowstride)
            .field("has_alpha", &self.has_alpha)
            .field("bits_per_sample", &self.bits_per_sample)
            .field("channels", &self.channels)
            .field("data (len)", &self.data.len())
            .finish()
    }
}

impl ImageData {
    pub fn to_texture(&self) -> MemoryTexture {
        let format = if self.has_alpha {
            gdk::MemoryFormat::R8g8b8a8
        } else {
            gdk::MemoryFormat::R8g8b8
        };

        gdk::MemoryTexture::new(
            self.width,
            self.height,
            format,
            &glib::Bytes::from(self.data.as_slice()),
            self.rowstride as usize,
        )
    }
}

impl From<HashMap<String, OwnedValue>> for NotificationHints {
    fn from(mut map: HashMap<String, OwnedValue>) -> Self {
        fn get_bool(
            map: &HashMap<String, OwnedValue>,
            key: &str,
        ) -> Option<bool> {
            map.get(key).and_then(|v| v.downcast_ref::<bool>().ok())
        }

        fn get_string(
            map: &HashMap<String, OwnedValue>,
            key: &str,
        ) -> Option<String> {
            map.get(key).and_then(|v| v.downcast_ref::<String>().ok())
        }

        fn get_i32(
            map: &HashMap<String, OwnedValue>,
            key: &str,
        ) -> Option<i32> {
            map.get(key).and_then(|v| v.downcast_ref::<i32>().ok())
        }

        fn get_u8(map: &HashMap<String, OwnedValue>, key: &str) -> Option<u8> {
            map.get(key).and_then(|v| v.downcast_ref::<u8>().ok())
        }

        fn get_image_data(
            map: &mut HashMap<String, OwnedValue>,
            key: &str,
        ) -> Option<ImageData> {
            ImageData::try_from(map.remove(key)?).ok()
        }

        Self {
            action_icons: get_bool(&map, "action-icons"),
            category: get_string(&map, "category"),
            desktop_entry: get_string(&map, "desktop-entry"),
            image_data: get_image_data(&mut map, "image-data"),
            image_path: get_string(&map, "image-path"),
            resident: get_bool(&map, "resident"),
            sound_file: get_string(&map, "sound-file"),
            sound_name: get_string(&map, "sound-name"),
            suppress_sound: get_bool(&map, "suppress-sound"),
            transient: get_bool(&map, "transient"),
            x: get_i32(&map, "x"),
            y: get_i32(&map, "y"),
            urgency: get_u8(&map, "urgency"),
        }
    }
}
