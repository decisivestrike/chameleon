use crate::notification::NotificationData;

/// Server action
pub enum Action {
    Create(NotificationData),
    Remove(u32),
}
