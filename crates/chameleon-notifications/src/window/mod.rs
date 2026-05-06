pub mod content;
pub use content::NotificationContent;

use gtk::prelude::*;
use gtk::{self, Window};
use gtke::WindowComponent;
use layer_shell::{Edge, LayerShell};

const NAMESPACE: &str = "chameleon-notifications";

#[derive(Clone, WindowComponent)]
pub struct NotificationWindow {
    pub content: NotificationContent,
    #[root]
    pub window: Window,
}

impl NotificationWindow {
    const NAME: &str = "notification-window";

    pub fn new(content: NotificationContent) -> Self {
        let window = Window::new();
        Self::setup_layershell(&window);

        window.set_child(Some(content.as_ref()));
        window.set_widget_name(Self::NAME);

        Self { content, window }
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
