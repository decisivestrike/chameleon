use crate::CONFIG;
use crate::notification::Notification;
use crate::requests::NotificationData;
use grapes::prelude::WidgetExt;
use indexmap::IndexMap;
use layer_shell::{Edge, LayerShell};
use tokio::sync::Mutex;

pub struct NotificationQueue {
    notifications: Mutex<IndexMap<u32, Notification>>,
    max_notifications: usize,
}

impl NotificationQueue {
    pub fn new() -> Self {
        let map = IndexMap::new();

        Self {
            notifications: Mutex::new(map).into(),
            max_notifications: CONFIG.max_notifications.get().into(),
        }
    }

    pub async fn push_or_replace(&self, data: NotificationData) {
        let mut notifications = self.notifications.lock().await;
        let maybe_notification = notifications.get_mut(&data.replaces_id);

        if let Some(old_notification) = maybe_notification {
            self.replace(old_notification, data).await
        } else {
            drop(notifications);
            self.push(data).await;
        }
    }

    pub async fn remove(&self, id: u32) {
        let mut notifications = self.notifications.lock().await;
        notifications.shift_remove(&id);
    }

    async fn push(&self, data: NotificationData) {
        let mut notifications = self.notifications.lock().await;
        let mut notifications_count = notifications.len();

        if notifications_count >= self.max_notifications.into() {
            notifications.shift_remove_index(0).expect("should exists");
            notifications_count -= 1;
        }

        for (index, entry) in notifications.iter().enumerate() {
            let notification = entry.1;

            let height = notification.window().height();
            let count = (notifications_count + 1 - index) as i32;

            let gaps = CONFIG.gaps as i32 * count;
            let heights = height * (count - 1);
            let margin_top = gaps + heights;

            notification.window().set_margin(Edge::Top, margin_top);
        }

        let id = data.replaces_id;
        let notification = Notification::new(data);
        notification.show();

        notifications.insert(id, notification);
    }

    async fn replace(&self, old: &mut Notification, data: NotificationData) {
        old.update(data);
    }
}

unsafe impl Sync for NotificationQueue {}
unsafe impl Send for NotificationQueue {}
