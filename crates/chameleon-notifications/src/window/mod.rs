pub mod content;
pub use content::NotificationContent;

use crate::config::WindowRules;
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{self};
use layer_shell::{Edge, LayerShell};

const NAMESPACE: &str = "chameleon-notifications";

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct NotificationWindowImp;

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationWindowImp {
        const NAME: &'static str = "NotificationWindow";
        type Type = super::NotificationWindow;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for NotificationWindowImp {}

    impl WidgetImpl for NotificationWindowImp {}

    impl WindowImpl for NotificationWindowImp {}
}

glib::wrapper! {
    pub struct NotificationWindow(ObjectSubclass<imp::NotificationWindowImp>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl NotificationWindow {
    const NAME: &str = "notification-window";

    pub fn new(
        content: NotificationContent,
        window_rules: &WindowRules,
    ) -> Self {
        let obj: Self = Object::builder().build();
        obj.setup_layershell(window_rules);
        obj.set_content(content);
        obj.set_widget_name(Self::NAME);
        obj
    }

    pub fn set_content(&self, content: NotificationContent) {
        self.set_child(Some(&content));
    }

    fn setup_layershell(&self, window_rules: &WindowRules) {
        self.init_layer_shell();
        self.set_namespace(Some(NAMESPACE));

        self.set_anchor(Edge::Top, true);
        self.set_margin(Edge::Top, window_rules.vgap.into());

        self.set_anchor(Edge::Right, true);
        self.set_margin(Edge::Right, window_rules.hgap.into());
    }
}
