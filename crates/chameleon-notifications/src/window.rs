use grapes::layer_shell::{Edge, LayerShell};
use grapes::prelude::{GtkWindowExt, WidgetExt};
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
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_size_request(224, 64);

        let title = gtk::Label::new(Some(&data.body));
        window.set_child(Some(&title));

        let window_clone = window.clone();
        glib::timeout_add_local_once(data.timeout, move || {
            window_clone.destroy();
        });

        Self { window }
    }
}
