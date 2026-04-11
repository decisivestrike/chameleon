use crate::{notification::Notification, requests::NotificationData};
use grapes::gtk;

pub struct NotificationFactory {
    app: gtk::Application,
}

impl NotificationFactory {
    pub fn new(app: &gtk::Application) -> Self {
        Self { app: app.clone() }
    }

    pub fn create(&self, id: u32, data: &NotificationData) -> Notification {
        Notification::new(&self.app, id, data)
    }
}
