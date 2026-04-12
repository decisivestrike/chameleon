use crate::{server::command::PushCommand, window::NotificationWindow};
use grapes::{
    WindowComponent,
    glib::{self, clone},
    gtk::{self, ApplicationWindow},
    prelude::GtkWindowExt,
};
use std::time::Duration;

pub struct Notification {
    id: u32,
    lifetime: Duration,
    window: NotificationWindow,
}

impl Notification {
    pub fn new(app: &gtk::Application, data: PushCommand) -> Self {
        let PushCommand {
            id,
            summary,
            body,
            lifetime,
        } = data;

        let window = NotificationWindow::new(&app);

        glib::timeout_add_local_once(
            lifetime,
            clone!(
                #[weak]
                window,
                move || window.destroy()
            ),
        );

        let notification = Self {
            id,
            window,
            lifetime,
        };

        notification.update(&summary, &body);

        notification
    }

    pub fn update(&self, summary: &String, body: &String) {
        let window = &self.window;

        window.summary.set_label(&summary);
        window.body.set_label(&body)
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
