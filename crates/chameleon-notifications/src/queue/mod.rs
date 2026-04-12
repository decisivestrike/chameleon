mod error;

use crate::{
    GAP, MAX_NOTIFICATIONS, notification::Notification,
    queue::error::QueueError,
};
use grapes::{
    layer_shell::{Edge, LayerShell},
    prelude::WidgetExt,
    tokio::sync::Mutex,
};
use indexmap::IndexMap;

pub struct NotificationQueue(Mutex<IndexMap<u32, Notification>>);

impl NotificationQueue {
    pub fn new() -> Self {
        let map = IndexMap::new();
        NotificationQueue(Mutex::new(map).into())
    }

    pub async fn push(
        &self,
        id: u32,
        new_notification: Notification,
    ) -> Result<(), QueueError> {
        let mut notifications = self.0.lock().await;
        if notifications.contains_key(&id) {
            return Err(QueueError::DuplicateId(id));
        }

        let mut notifications_count = notifications.len();
        if notifications_count >= MAX_NOTIFICATIONS {
            notifications.shift_remove_index(0).expect("should exists");
            notifications_count -= 1;
        }

        for (index, entry) in notifications.iter().enumerate() {
            let notification = entry.1;

            let height = notification.window().height();
            let count = (notifications_count + 1 - index) as i32;

            let gaps = GAP * count;
            let heights = height * (count - 1);
            let margin_top = gaps + heights;

            notification.window().set_margin(Edge::Top, margin_top);
        }

        notifications.insert(id, new_notification);

        Ok(())
    }

    pub async fn modify(&self, id: u32, f: impl Fn(&mut Notification)) {
        let mut notifications = self.0.lock().await;

        if let Some(notification) = notifications.get_mut(&id) {
            f(notification)
        }
    }

    pub async fn remove(&self, id: u32) {
        let mut notifications = self.0.lock().await;
        notifications.shift_remove(&id);
    }
}

unsafe impl Sync for NotificationQueue {}
unsafe impl Send for NotificationQueue {}
