use crate::{GAP, MAX_NOTIFICATIONS, window::NotificationWindow};
use grapes::{
    WindowComponent,
    layer_shell::{Edge, LayerShell},
    prelude::WidgetExt,
    tokio::sync::Mutex,
};
use std::collections::LinkedList;

pub static NOTIFICATION_WINDOWS: NotificationList =
    NotificationList::const_new();

pub struct NotificationList(Mutex<LinkedList<NotificationWindow>>);

impl NotificationList {
    pub const fn const_new() -> Self {
        NotificationList(Mutex::const_new(LinkedList::new()))
    }

    pub async fn append(&self, new_window: NotificationWindow) {
        let mut notification_windows = self.0.lock().await;
        let mut notifications_count = notification_windows.len();

        if notifications_count >= MAX_NOTIFICATIONS {
            let displaced = notification_windows.pop_front().unwrap();
            displaced.destroy();

            notifications_count -= 1;
        }

        for (index, notification) in notification_windows.iter().enumerate() {
            let height = notification.window.height();
            let count = (notifications_count + 1 - index) as i32;

            let gaps = GAP * count;
            let heights = height * (count - 1);

            let margin_top: i32 = gaps + heights;

            notification.window.set_margin(Edge::Top, margin_top);
        }

        notification_windows.push_back(new_window);
    }

    pub async fn remove(&self, id: u32) {
        let mut notification_windows = self.0.lock().await;

        let maybe_index =
            notification_windows.iter().position(|w| w.id() == id);

        if let Some(index) = maybe_index {
            let mut tail = notification_windows.split_off(index);
            let popped = tail.pop_front().unwrap();
            notification_windows.append(&mut tail);

            popped.destroy();
        }
    }
}

unsafe impl Sync for NotificationList {}
unsafe impl Send for NotificationList {}
