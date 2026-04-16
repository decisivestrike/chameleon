use crate::requests::NotificationData;
use crate::requests::notification_data::hints::NotificationHints;
use std::time::Duration;

#[derive(Debug)]
pub struct NotificationCommand {
    pub id: u32,
    pub summary: String,
    pub body: String,
    pub icon_path: String,
    pub hints: NotificationHints,
    pub lifetime: Duration,
}

impl NotificationCommand {
    pub fn new(id: u32, data: NotificationData) -> Self {
        let NotificationData {
            app_icon,
            summary,
            body,
            hints,
            expire_timeout,
            ..
        } = data;

        Self {
            id,
            summary,
            body,
            icon_path: app_icon,
            hints,
            lifetime: expire_timeout.into(),
        }
    }
}
