use crate::{GAP, MAX_NOTIFICATIONS, notification::Notification};
use grapes::{
    layer_shell::{Edge, LayerShell},
    prelude::WidgetExt,
    tokio::sync::Mutex,
};
use std::collections::LinkedList;

pub struct NotificationQueue(Mutex<LinkedList<Notification>>);

impl NotificationQueue {
    pub const fn const_new() -> Self {
        NotificationQueue(Mutex::const_new(LinkedList::new()))
    }

    pub async fn append(&self, new_window: Notification) {
        let mut notification_windows = self.0.lock().await;
        let mut notifications_count = notification_windows.len();

        if notifications_count >= MAX_NOTIFICATIONS {
            notification_windows.pop_front().expect("should exists");
            notifications_count -= 1;
        }

        for (index, notification) in notification_windows.iter().enumerate() {
            let height = notification.window().height();
            let count = (notifications_count + 1 - index) as i32;

            let gaps = GAP * count;
            let heights = height * (count - 1);
            let margin_top = gaps + heights;

            notification.window().set_margin(Edge::Top, margin_top);
        }

        notification_windows.push_back(new_window);
    }

    pub async fn remove(&self, id: u32) {
        let mut notification_windows = self.0.lock().await;

        let maybe_index =
            notification_windows.iter().position(|w| w.id() == id);

        if let Some(index) = maybe_index {
            let mut tail = notification_windows.split_off(index);
            tail.pop_front().expect("should exists");
            notification_windows.append(&mut tail);
        }
    }
}

unsafe impl Sync for NotificationQueue {}
unsafe impl Send for NotificationQueue {}
