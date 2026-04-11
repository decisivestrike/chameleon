use crate::{
    DEFAULT_TIMEOUT, NOTIFICATION_QUEUE, requests::NotificationData,
    window::NotificationWindow,
};
use grapes::{
    WindowComponent,
    glib::{self, clone},
    gtk::{self, ApplicationWindow},
    prelude::{GtkWindowExt, WidgetExt},
};
use std::time::Duration;

pub struct Notification {
    id: u32,
    window: NotificationWindow,
}

impl Notification {
    pub fn new(
        app: &gtk::Application,
        id: u32,
        data: &NotificationData,
    ) -> Self {
        let window = NotificationWindow::new(&app);
        let lifetime = Self::timeout_to_duration(data.expire_timeout);

        glib::timeout_add_local_once(
            lifetime,
            clone!(
                #[weak]
                window,
                move || {
                    window.destroy();
                    log::debug!("notification #{id} destroyed");
                }
            ),
        );

        let notification = Self { id, window };
        notification.window().connect_destroy({
            move |_| {
                glib::spawn_future_local(NOTIFICATION_QUEUE.remove(id));
            }
        });
        notification.update(data);

        notification
    }

    pub fn update(&self, data: &NotificationData) {
        let window = &self.window;

        window.summary.set_label(&data.summary);
        window.body.set_label(&data.body)
    }

    pub fn show(&self) {
        self.window().present();
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn window(&self) -> ApplicationWindow {
        self.window.window.clone()
    }

    fn timeout_to_duration(expire_timeout: i32) -> Duration {
        match expire_timeout {
            timeout if timeout < 0 => DEFAULT_TIMEOUT,
            timeout if timeout == 0 => Duration::MAX,
            timeout => Duration::from_millis(timeout as u64),
        }
    }
}

impl Drop for Notification {
    fn drop(&mut self) {
        self.window.destroy();
    }
}
