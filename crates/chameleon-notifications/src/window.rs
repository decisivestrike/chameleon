use grapes::gtk::Orientation;
use grapes::layer_shell::{Edge, LayerShell};
use grapes::prelude::{BoxExt, GtkWindowExt, WidgetExt};
use grapes::{WindowComponent, gtk::ApplicationWindow};
use grapes::{glib, gtk};

use crate::notifications::NotificationData;

#[derive(WindowComponent)]
pub struct NotificationWindow {
    #[root]
    window: ApplicationWindow,
}

impl NotificationWindow {
    pub fn new(app: &gtk::Application, data: NotificationData) -> Self {
        let window = ApplicationWindow::new(app);

        window.init_layer_shell();
        window.set_size_request(224, 64);

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, 10);

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, 10);

        let title = gtk::Label::new(Some(&data.title));
        let body = gtk::Label::new(Some(&data.body));

        let container = gtk::Box::new(Orientation::Vertical, 10);
        container.append(&title);
        container.append(&body);

        window.set_child(Some(&container));

        let window_clone = window.clone();
        glib::timeout_add_local_once(data.timeout, move || {
            log::debug!("notification destroyed");
            window_clone.destroy();
        });

        Self { window }
    }
}
