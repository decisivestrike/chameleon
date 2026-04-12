use crate::GAP;
use grapes::WindowComponent;
use gtk::{self, ApplicationWindow, Orientation, glib, prelude::*};
use layer_shell::{Edge, LayerShell};

#[derive(Clone, glib::Downgrade, WindowComponent)]
pub struct NotificationWindow {
    pub icon: gtk::Image,
    pub summary: gtk::Label,
    pub body: gtk::Label,
    #[root]
    pub window: ApplicationWindow,
}

impl NotificationWindow {
    pub fn new(app: &gtk::Application) -> Self {
        let window = ApplicationWindow::new(app);
        Self::setup_layershell(&window);

        let icon = gtk::Image::new();
        let title = gtk::Label::builder().name("summary").build();
        let body = gtk::Label::builder().name("body").build();

        let container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .name("notification")
            .build();

        container.append(&icon);
        container.append(&title);
        container.append(&body);

        window.set_child(Some(&container));
        window.set_widget_name("notification-window");

        Self {
            summary: title,
            icon,
            body,
            window,
        }
    }

    fn setup_layershell(window: &ApplicationWindow) {
        window.init_layer_shell();
        window.set_namespace(Some("chameleon-notifications"));

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, GAP);

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, GAP);
    }
}
