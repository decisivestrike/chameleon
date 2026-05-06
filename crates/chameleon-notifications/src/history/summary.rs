use crate::notification::NotificationData;
use gtk::gdk;

#[derive(Debug, Default)]
pub struct Summary {
    /// The optional name of the application sending the notification. Can be
    /// blank.
    pub app_name: String,

    /// Texture
    pub icon_texture: Option<gdk::MemoryTexture>,

    /// Title
    pub title: String,

    /// The optional detailed body text. Can be empty.
    pub body: String,
}

impl Summary {
    pub fn from_data(data: NotificationData) -> Option<Self> {
        if let Some(true) = data.hints.transient {
            None
        } else {
            Some(Self {
                app_name: data.app_name,
                icon_texture: data.hints.image_data.map(|data| (&data).into()),
                title: data.title,
                body: data.body,
            })
        }
    }
}
