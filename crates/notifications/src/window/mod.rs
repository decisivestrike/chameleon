pub mod content;
pub use content::NotificationContent;

use crate::notification::Urgency;
use gtk::gdk::MemoryTexture;
use gtk::prelude::*;
use gtk::{self, Orientation, Window};
use gtke::WindowComponent;
use layer_shell::{Edge, LayerShell};

const NAMESPACE: &str = "chameleon-notifications";

#[derive(Clone, WindowComponent)]
pub struct NotificationWindow {
    pub icon: Option<gtk::Image>,
    pub summary: gtk::Label,
    pub body: gtk::Label,

    pub text_container: gtk::Box,
    pub container: gtk::Box,
    #[root]
    pub window: Window,
}

impl NotificationWindow {
    const NAME: &str = "notification-window";

    pub fn new(urgency: Option<Urgency>) -> Self {
        let window = Window::new();
        Self::setup_layershell(&window);

        let title = gtk::Label::builder().name("summary").build();
        let body = gtk::Label::builder().name("body").build();

        let urgency = urgency.unwrap_or(Urgency::Low);
        let urgency_class = urgency.as_ref();

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
        window.set_widget_name(Self::NAME);

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
                .pixel_size(32 /* CONFIG.icon_size.into() */)
                .build();

            self.container.prepend(&icon);
            self.icon = Some(icon);
        }
    }

    fn setup_layershell(window: &Window) {
        let gap = 10; // CONFIG.gaps.into();

        window.init_layer_shell();
        window.set_namespace(Some(NAMESPACE));

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, gap);

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, 10);
    }
}
