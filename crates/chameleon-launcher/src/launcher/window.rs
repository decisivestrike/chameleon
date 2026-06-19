use crate::config::LauncherConfig;
use crate::launcher::content::LauncherContent;
use crate::providers::Direction;
use gtk::gdk::Key;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, Propagation, clone};
use gtk::prelude::*;
use gtk::{EventControllerKey, PropagationPhase, glib};
use layer_shell::{KeyboardMode, Layer, LayerShell};

mod imp {
    use super::*;
    use crate::launcher::content::LauncherContent;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct LauncherImp {
        pub(crate) root: OnceLock<LauncherContent>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherImp {
        const NAME: &'static str = "ChameleonLauncher";
        type Type = super::Launcher;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for LauncherImp {
        fn constructed(&self) {
            self.parent_constructed();

            let window = self.obj();

            window.set_focusable(true);
            window.set_vexpand(false);
            window.set_hexpand(false);
            window.set_default_size(-1, -1); // Auto size
            window.set_widget_name("launcher-window");

            window.init_layer_shell();
            window.set_namespace(Some("chameleon-launcher"));
            window.set_layer(Layer::Top);
            window.set_keyboard_mode(KeyboardMode::OnDemand);
        }
    }

    impl WidgetImpl for LauncherImp {}

    impl WindowImpl for LauncherImp {}
}

glib::wrapper! {
    pub struct Launcher(ObjectSubclass<imp::LauncherImp>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Launcher {
    pub fn new(config: LauncherConfig) -> Self {
        let launcher: Self = Object::builder().build();
        launcher.init(config);

        let root_controller = EventControllerKey::new();
        root_controller.connect_key_pressed(clone!(
            #[strong]
            launcher,
            move |_, key, _, _| launcher.handle_keypress(&key)
        ));
        root_controller.set_propagation_phase(PropagationPhase::Capture);
        launcher.add_controller(root_controller);

        launcher
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.get_visible();

        if target_visibility {
            self.present();
        } else {
            self.set_visible(target_visibility);
            self.root().reset_state();
        }
    }

    fn handle_keypress(&self, key: &Key) -> Propagation {
        match *key {
            key @ (Key::Up | Key::Right | Key::Down | Key::Left) => {
                let direction = key.try_into().expect("already checked");
                self.move_selection(direction);
                Propagation::Stop
            }
            Key::Tab => {
                self.next_provider();
                Propagation::Stop
            }
            Key::Return => {
                self.invoke_action();
                Propagation::Stop
            }
            Key::Escape => {
                self.toggle_visibility();
                Propagation::Stop
            }
            _ => Propagation::Proceed,
        }
    }

    fn init(&self, config: LauncherConfig) {
        let content = LauncherContent::new(config);
        self.set_child(Some(&content));
        self.imp()
            .root
            .set(content)
            .expect("can't be already initialized");
    }

    fn root(&self) -> &LauncherContent {
        self.imp().root.get().expect("must be initialized")
    }

    fn move_selection(&self, direction: Direction) {
        self.root().move_selection(direction)
    }

    fn next_provider(&self) {
        self.root().imp().switcher.next_page();
    }

    fn invoke_action(&self) {
        let invoked = self.root().invoke_provider_action();

        if invoked {
            self.toggle_visibility()
        }
    }
}
