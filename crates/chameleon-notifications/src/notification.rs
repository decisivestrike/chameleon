use crate::server::NotificationCommand;
use crate::window::NotificationWindow;
use grapes::WindowComponent;
use grapes::glib::{self, clone};
use grapes::gtk::{self, ApplicationWindow};
use grapes::prelude::GtkWindowExt;
use gtk::gdk::MemoryTexture;
use std::time::Duration;

pub struct Notification {
    id: u32,
    lifetime: Duration,
    window: NotificationWindow,
}

impl Notification {
    pub fn new(app: &gtk::Application, data: NotificationCommand) -> Self {
        let NotificationCommand { id, lifetime, .. } = data;
        let window = NotificationWindow::new(&app, data.hints.urgency);

        glib::timeout_add_local_once(
            lifetime,
            clone!(
                #[strong]
                window,
                move || window.destroy()
            ),
        );

        let mut notification = Self {
            id,
            window,
            lifetime,
        };

        notification.update(data);

        notification
    }

    pub fn update(&mut self, data: NotificationCommand) {
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

    pub fn lifetime(&self) -> Duration {
        self.lifetime
    }

    pub fn window(&self) -> ApplicationWindow {
        self.window.window.clone()
    }
}

impl Drop for Notification {
    fn drop(&mut self) {
        self.window.destroy();
    }
}
