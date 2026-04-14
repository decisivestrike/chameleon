use crate::requests::notification_data::Urgency;
use crate::server::image_data::ImageData;
use zbus::zvariant::{DeserializeDict, Type};

/// Hints are a way to provide extra data to a notification server that the
/// server may be able to make use of.
#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "a{sv}", rename_all = "kebab-case")]
pub struct NotificationHints {
    /// When set, a server that has the "action-icons" capability will attempt
    /// to interpret any action identifier as a named icon. The localized
    /// display name will be used to annotate the icon for accessibility
    /// purposes. The icon name should be compliant with the
    /// Freedesktop.org Icon Naming Specification.
    pub action_icons: Option<bool>,

    /// The type of notification this is.
    pub category: Option<String>,

    /// This specifies the name of the desktop filename representing the
    /// calling program. This should be the same as the prefix used for the
    /// application's .desktop file. An example would be "rhythmbox" from
    /// "rhythmbox.desktop". This can be used by the daemon to retrieve the
    /// correct icon for the application, for logging purposes, etc.
    pub desktop_entry: Option<String>,

    /// This is a raw data image format which describes the width, height,
    /// rowstride, has alpha, bits per sample, channels and image data
    /// respectively.
    pub image_data: Option<ImageData>,

    /// Alternative way to define the notification image.
    pub image_path: Option<String>,

    /// When set the server will not automatically remove the notification when
    /// an action has been invoked. The notification will remain resident in
    /// the server until it is explicitly removed by the user or by the sender.
    /// This hint is likely only useful when the server has the "persistence"
    /// capability.
    pub resident: Option<bool>,

    /// The path to a sound file to play when the notification pops up.
    pub sound_file: Option<String>,

    /// A themeable named sound from the freedesktop.org sound naming
    /// specification to play when the notification pops up. Similar to
    /// icon-name, only for sounds. An example would be "message-new-instant".
    pub sound_name: Option<String>,

    /// Causes the server to suppress playing any sounds, if it has that
    /// ability. This is usually set when the client itself is going to play
    /// its own sound.
    pub suppress_sound: Option<bool>,

    /// When set the server will treat the notification as transient and
    /// by-pass the server's persistence capability, if it should exist.
    pub transient: Option<bool>,

    /// Specifies the X location on the screen that the notification should
    /// point to. The "y" hint must also be specified.
    pub x: Option<i32>,

    /// Specifies the Y location on the screen that the notification should
    /// point to. The "x" hint must also be specified.
    pub y: Option<i32>,

    /// The urgency level.
    pub urgency: Option<Urgency>,
}
