use crate::{
    DEFAULT_TIMEOUT, requests::NotificationData,
    server::hints::NotificationHints,
};
use std::time::Duration;

pub enum NotificationCommand {
    Push(PushCommand),
    Replace(ReplaceCommand),
}

pub struct PushCommand {
    pub id: u32,
    pub summary: String,
    pub body: String,
    pub icon_path: String,
    pub hints: NotificationHints,
    pub lifetime: Duration,
}

pub struct ReplaceCommand {
    pub id: u32,
    pub summary: String,
    pub body: String,
}

impl NotificationCommand {
    pub fn new(id: u32, data: NotificationData, replace: bool) -> Self {
        if replace {
            NotificationCommand::replace(id, data)
        } else {
            NotificationCommand::push(id, data)
        }
    }

    fn push(id: u32, data: NotificationData) -> Self {
        let NotificationData {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        } = data;

        let lifetime = Self::timeout_to_duration(expire_timeout);

        let push_command = PushCommand {
            id,
            summary,
            body,
            icon_path: app_icon,
            hints: NotificationHints::from(hints),
            lifetime,
        };

        NotificationCommand::Push(push_command)
    }

    fn replace(id: u32, data: NotificationData) -> Self {
        let NotificationData {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        } = data;

        let replace_command = ReplaceCommand { id, summary, body };

        NotificationCommand::Replace(replace_command)
    }

    fn timeout_to_duration(expire_timeout: i32) -> Duration {
        match expire_timeout {
            timeout if timeout < 0 => DEFAULT_TIMEOUT,
            timeout if timeout == 0 => Duration::MAX,
            timeout => Duration::from_millis(timeout as u64),
        }
    }
}
