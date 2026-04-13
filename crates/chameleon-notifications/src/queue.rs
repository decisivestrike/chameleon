use crate::notification::Notification;
use crate::server::NotificationCommand;
use crate::{GAP, MAX_NOTIFICATIONS};
use grapes::prelude::WidgetExt;
use indexmap::IndexMap;
use layer_shell::{Edge, LayerShell};
use tokio::sync::Mutex;

pub struct NotificationQueue {
    app: gtk::Application,
    notifications: Mutex<IndexMap<u32, Notification>>,
}

impl NotificationQueue {
    pub fn new(app: &gtk::Application) -> Self {
        let map = IndexMap::new();

        Self {
            app: app.clone(),
            notifications: Mutex::new(map).into(),
        }
    }

    pub async fn push_or_replace(&self, new_notification: NotificationCommand) {
        let mut notifications = self.notifications.lock().await;
        let maybe_notification = notifications.get_mut(&new_notification.id);

        if let Some(old_notification) = maybe_notification {
            self.replace(old_notification, new_notification).await
        } else {
            drop(notifications);
            self.push(new_notification).await;
        }
    }

    pub async fn remove(&self, id: u32) {
        let mut notifications = self.notifications.lock().await;
        notifications.shift_remove(&id);
    }

    async fn push(&self, command: NotificationCommand) {
        let mut notifications = self.notifications.lock().await;
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

        let id = command.id;
        let notification = Notification::new(&self.app, command);
        notification.show();

        notifications.insert(id, notification);
    }

    async fn replace(
        &self,
        old: &mut Notification,
        command: NotificationCommand,
    ) {
        old.update(command);
    }
}

unsafe impl Sync for NotificationQueue {}
unsafe impl Send for NotificationQueue {}
