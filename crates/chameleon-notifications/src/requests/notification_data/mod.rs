//! https://specifications.freedesktop.org/notification/latest/protocol.html#command-notify
pub mod hints;
pub use hints::NotificationHints;

pub mod urgency;
pub use urgency::Urgency;

use serde::Deserialize;
use zbus::zvariant::Type;

#[derive(Debug, Deserialize, Type)]
pub struct NotificationData {
    /// The optional name of the application sending the notification. Can be blank.
    pub app_name: String,

    /// The optional notification ID that this notification replaces.
    /// The server must atomically (ie with no flicker or other visual cues)
    /// replace the given notification with this one. This allows clients to effectively
    /// modify the notification while it's active. A value of value of 0 means that
    /// this notification won't replace any existing notifications.
    pub replaces_id: u32,

    /// The optional program icon of the calling application.
    /// See Icons and Images. Can be an empty string, indicating no icon.
    pub app_icon: String,

    /// The summary text briefly describing the notification.
    pub summary: String,

    /// The optional detailed body text. Can be empty.
    pub body: String,

    /// Actions are sent over as a list of pairs.
    /// Each even element in the list (starting at index 0) represents the identifier for the action.
    /// Each odd element in the list is the localized string that will be displayed to the user.
    pub actions: Vec<String>,

    /// Optional hints that can be passed to the server from the client program.
    /// Although clients and servers should never assume each other supports any specific hints,
    /// they can be used to pass along information, such as the process PID or window ID,
    /// that the server may be able to make use of. See Hints. Can be empty.
    pub hints: NotificationHints,

    /// The timeout time in milliseconds since the display of the notification at
    /// which the notification should automatically close.
    /// If -1, the notification's expiration time is dependent on the notification
    /// server's settings, and may vary for the type of notification. If 0, never expire.
    pub expire_timeout: i32,
}
