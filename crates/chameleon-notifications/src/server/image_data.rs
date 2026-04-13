use gtk::gdk::{self, MemoryTexture};
use gtk::glib;
use serde::Deserialize;
use std::fmt;
use zbus::zvariant::{OwnedValue, Type};

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

impl From<&ImageData> for MemoryTexture {
    fn from(image_data: &ImageData) -> Self {
        let format = if image_data.has_alpha {
            gdk::MemoryFormat::R8g8b8a8
        } else {
            gdk::MemoryFormat::R8g8b8
        };

        gdk::MemoryTexture::new(
            image_data.width,
            image_data.height,
            format,
            &glib::Bytes::from(image_data.data.as_slice()),
            image_data.rowstride as usize,
        )
    }
}
