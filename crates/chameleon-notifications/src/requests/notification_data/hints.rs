use crate::{
    requests::notification_data::urgency::Urgency,
    server::image_data::ImageData,
};
use zbus::zvariant::{DeserializeDict, Type};

/// Hints are a way to provide extra data to a notification server that the server may be able to make use of.
#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "a{sv}")]
pub struct NotificationHints {
    /// When set, a server that has the "action-icons" capability will attempt to
    /// interpret any action identifier as a named icon. The localized display name
    /// will be used to annotate the icon for accessibility purposes. The icon name
    /// should be compliant with the Freedesktop.org Icon Naming Specification.
    pub action_icons: Option<bool>,

    /// The type of notification this is.
    pub category: Option<String>,

    /// This specifies the name of the desktop filename representing the calling program.
    /// This should be the same as the prefix used for the application's .desktop file.
    /// An example would be "rhythmbox" from "rhythmbox.desktop". This can be used by
    /// the daemon to retrieve the correct icon for the application, for logging purposes, etc.
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

    /// The urgency level.
    pub urgency: Option<Urgency>,
}
