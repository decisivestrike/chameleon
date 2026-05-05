pub mod data;
pub use data::NotificationData;

pub mod hints;
pub use hints::NotificationHints;

pub mod image_data;
pub use image_data::ImageData;

pub mod timeout;
pub use timeout::Timeout;

pub mod urgency;
pub use urgency::Urgency;

use crate::window::NotificationWindow;
use gtk::Window;
use gtk::gdk::MemoryTexture;
use gtk::glib::clone;
use gtk::prelude::GtkWindowExt;
use gtke::WindowComponent;
use gtkio::time::timeout_local;

pub struct Notification {
    id: u32,
    window: NotificationWindow,
}

impl Notification {
    pub fn new(data: &NotificationData) -> Self {
        let NotificationData {
            replaces_id,
            expire_timeout,
            ..
        } = data;

        let window = NotificationWindow::new(data.hints.urgency);

        timeout_local(
            (*expire_timeout).into(),
            clone!(
                #[strong]
                window,
                move || window.destroy()
            ),
        );

        let mut notification = Self {
            id: *replaces_id,
            window,
        };

        notification.update(data);

        notification
    }

    pub fn update(&mut self, data: &NotificationData) {
        let window = &mut self.window;

        if let Some(ref image_data) = data.hints.image_data {
            let texture = MemoryTexture::from(image_data);
            window.add_icon(texture);
        }

        match &data.hints.desktop_entry {
            Some(entry_name) => window
                .summary
                .set_label(&format!("[{entry_name}] {}", data.title)),
            None => window.summary.set_label(&data.title),
        };

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
