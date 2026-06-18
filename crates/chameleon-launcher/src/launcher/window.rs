use crate::config::LauncherConfig;
use crate::launcher::content::LauncherContent;
use gtk::gdk::Key;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
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
            window.set_keyboard_mode(KeyboardMode::Exclusive);
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
            move |_, key, _, _| {
                match key {
                    Key::Left => glib::Propagation::Stop,
                    // Key::Down => {
                    //     if let Some(provider) = launcher.active_provider() {
                    //         provider.select_below();
                    //     }
                    //     glib::Propagation::Stop
                    // }
                    Key::Tab => {
                        launcher.next_provider();
                        glib::Propagation::Stop
                    }
                    Key::Return => {
                        launcher.invoke_action();
                        glib::Propagation::Stop
                    }
                    Key::Escape => {
                        launcher.toggle_visibility();
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
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

    fn update_model(&self) {
        self.root().update_model_from_query();
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

    fn invoke_action(&self) {
        let invoked = self.root().invoke_provider_action();

        if invoked {
            self.toggle_visibility()
        }
    }

    fn next_provider(&self) {
        self.root().imp().switcher.next_provider();
    }
}
