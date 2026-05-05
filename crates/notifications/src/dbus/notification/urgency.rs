use serde_repr::Deserialize_repr;
use zbus::zvariant::Type;

/// Notifications have an urgency level associated with them.
/// This defines the importance of the notification. For example,
/// "Joe Bob signed on" would be a low urgency. "You have new mail" or "A USB
/// device was unplugged" would be a normal urgency. "Your computer is on fire"
/// would be a critical urgency.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize_repr, PartialEq, PartialOrd, Type)]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

impl AsRef<str> for Urgency {
    fn as_ref(&self) -> &str {
        match self {
            Urgency::Low => "low",
            Urgency::Normal => "normal",
            Urgency::Critical => "critical",
        }
    }
}
