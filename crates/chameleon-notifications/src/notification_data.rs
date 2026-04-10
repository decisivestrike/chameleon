use std::{collections::HashMap, time::Duration};

use crate::DEFAULT_TIMEOUT;

#[derive(Clone)]
pub struct NotificationData {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub lifetime: Duration,
}

impl NotificationData {
    pub fn new(
        id: u32,
        _app_name: &str,
        _app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<String>,
        _hints: HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> Self {
        let lifetime = match expire_timeout {
            timeout if timeout < 0 => DEFAULT_TIMEOUT,
            timeout if timeout == 0 => Duration::MAX,
            timeout => Duration::from_millis(timeout as u64),
        };

        Self {
            id,
            title: summary.to_string(),
            body: body.to_string(),
            lifetime,
        }
    }
}
