use crate::requests::NotificationData;
use crate::window::NotificationWindow;
use grapes::WindowComponent;
use grapes::glib::clone;
use grapes::gtk::{self};
use grapes::prelude::GtkWindowExt;
use gtk::Window;
use gtk::gdk::MemoryTexture;
use gtkio::time::timeout_local;

pub struct Notification {
    id: u32,
    window: NotificationWindow,
}

impl Notification {
    pub fn new(data: NotificationData) -> Self {
        let NotificationData {
            replaces_id,
            expire_timeout,
            ..
        } = data;
        let window = NotificationWindow::new(data.hints.urgency);

        timeout_local(
            expire_timeout.into(),
            clone!(
                #[strong]
                window,
                move || window.destroy()
            ),
        );

        let mut notification = Self {
            id: replaces_id,
            window,
        };

        notification.update(data);

        notification
    }

    pub fn update(&mut self, data: NotificationData) {
        let window = &mut self.window;

        if let Some(ref image_data) = data.hints.image_data {
            let texture = MemoryTexture::from(image_data);
            window.add_icon(texture);
        }

        let summary = match data.hints.desktop_entry {
            Some(entry_name) => format!("[{entry_name}] {}", data.summary),
            None => data.summary,
        };

        window.summary.set_label(&summary);
        window.body.set_label(&data.body)
    }

    pub fn show(&self) {
        self.window().present();
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn window(&self) -> Window {
        self.window.window.clone()
    }
}

impl Drop for Notification {
    fn drop(&mut self) {
        self.window.destroy();
    }
}
