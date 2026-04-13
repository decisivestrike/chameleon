use crate::{GAP, ICON_SIZE};
use grapes::WindowComponent;
use gtk::{
    self, ApplicationWindow, Orientation, gdk::MemoryTexture, prelude::*,
};
use layer_shell::{Edge, LayerShell};

#[derive(Clone, WindowComponent)]
pub struct NotificationWindow {
    pub icon: Option<gtk::Image>,
    pub summary: gtk::Label,
    pub body: gtk::Label,

    pub text_container: gtk::Box,
    pub container: gtk::Box,
    #[root]
    pub window: ApplicationWindow,
}

impl NotificationWindow {
    pub fn new(app: &gtk::Application, urgency: Option<u8>) -> Self {
        let window = ApplicationWindow::new(app);
        Self::setup_layershell(&window);

        let title = gtk::Label::builder().name("summary").build();
        let body = gtk::Label::builder().name("body").build();

        let urgency_class = match urgency {
            Some(level) => match level {
                1 => "low",
                2 => "normal",
                3 => "critical",
                another_level => {
                    log::error!("Uncorrect urgency level: {another_level}");
                    "low"
                }
            },
            None => "low",
        };

        let text_container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();

        text_container.append(&title);
        text_container.append(&body);

        let container = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .name("notification")
            .css_classes([urgency_class])
            .build();

        container.append(&text_container);

        window.set_child(Some(&container));
        window.set_widget_name("notification-window");

        Self {
            summary: title,
            icon: None,
            body,
            text_container,
            container,
            window,
        }
    }

    pub fn add_icon(&mut self, texture: MemoryTexture) {
        if let Some(ref icon) = self.icon {
            icon.set_paintable(Some(&texture));
        } else {
            let icon = gtk::Image::builder()
                .paintable(&texture)
                .pixel_size(ICON_SIZE)
                .build();

            self.container.prepend(&icon);
            self.icon = Some(icon);
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
