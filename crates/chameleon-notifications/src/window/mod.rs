pub mod content;
pub use content::NotificationContent;

use crate::config::WindowRules;
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

    pub fn new(
        content: NotificationContent,
        window_rules: &WindowRules,
    ) -> Self {
        let window = Window::new();
        Self::setup_layershell(&window, window_rules);

        window.set_child(Some(content.as_ref()));
        window.set_widget_name(Self::NAME);

        Self { content, window }
    }

    pub fn set_content(&self, content: NotificationContent) {
        self.window.set_child(Some(content.as_ref()));
    }

    pub fn window(&self) -> gtk::Window {
        self.window.clone()
    }

    fn setup_layershell(window: &Window, window_rules: &WindowRules) {
        window.init_layer_shell();
        window.set_namespace(Some(NAMESPACE));

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, window_rules.vgap.into());

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, window_rules.hgap.into());
    }
}
