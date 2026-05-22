use crate::config::LauncherConfig;
use crate::providers::Provider;
use gtk::gdk::Key;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::{
    Align, EventControllerKey, PolicyType, PropagationPhase, ScrolledWindow,
    glib,
};
use layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

mod imp {
    use super::*;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Launcher)]
    pub struct LauncherImp {
        pub searchbar: gtk::Entry,
        pub stack: gtk::Stack,
        pub providers: RefCell<HashMap<String, Rc<dyn Provider>>>,

        #[property(get, set)]
        active_provider_name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherImp {
        const NAME: &'static str = "ChameleonLauncher";
        type Type = super::Launcher;
        type ParentType = gtk::Window;
    }

    #[glib::derived_properties]
    impl ObjectImpl for LauncherImp {
        fn constructed(&self) {
            self.parent_constructed();

            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .valign(gtk::Align::Start)
                .hexpand(false)
                .vexpand(false)
                .homogeneous(false)
                .spacing(0)
                .build();

            container.append(&self.searchbar);
            container.append(&self.stack);

            let obj = self.obj();
            obj.init_layer_shell();
            obj.set_default_size(-1, -1); // Auto size
            obj.set_vexpand(false);
            obj.set_hexpand(false);
            obj.set_widget_name("launcher");
            obj.set_namespace(Some("chameleon-launcher"));

            // TODO: Make configurable
            obj.set_layer(Layer::Top);
            obj.set_keyboard_mode(if true {
                KeyboardMode::Exclusive
            } else {
                KeyboardMode::OnDemand
            });

            obj.set_child(Some(&container));
            obj.set_focusable(true);
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

        launcher
            .bind_property(
                "active-provider-name",
                &launcher.imp().stack,
                "visible-child-name",
            )
            .bidirectional()
            .sync_create()
            .build();

        let imp = launcher.imp();
        imp.searchbar
            .set_placeholder_text(Some(&config.searchbar_placeholder));

        let providers = config.instantiate_providers();

        for p in providers.into_iter() {
            let scrolled_window = ScrolledWindow::builder()
                .propagate_natural_height(true)
                .valign(Align::Start)
                .hscrollbar_policy(PolicyType::Never)
                .vscrollbar_policy(PolicyType::Automatic)
                .can_focus(false)
                .can_target(false)
                .min_content_width(480)
                .max_content_width(720)
                .min_content_height(0)
                .max_content_height(420)
                .hexpand(false)
                .vexpand(false)
                .overlay_scrolling(true)
                .child(&p.view())
                .build();

            launcher
                .imp()
                .stack
                .add_named(&scrolled_window, Some(p.name()));

            launcher
                .imp()
                .providers
                .borrow_mut()
                .insert(p.name().to_string(), p);
        }

        // Config
        imp.stack.set_visible_child_name("applications");

        // On enter hit
        let _handler_id = imp.searchbar.connect_activate(clone!(
            #[strong]
            launcher,
            move |_| {
                launcher.active_provider().map(|p| p.invoke_action()).map(
                    |invoked| {
                        if invoked {
                            launcher.toggle_visibility()
                        }
                    },
                );
            }
        ));

        // Exit on esc
        let esc_controller = EventControllerKey::new();
        esc_controller.connect_key_pressed(clone!(
            #[strong]
            launcher,
            move |_, key, _, _| {
                if key == Key::Escape {
                    launcher.toggle_visibility();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        esc_controller.set_propagation_phase(PropagationPhase::Capture);
        launcher.add_controller(esc_controller);

        // Initial
        launcher.active_provider().map(|p| p.update_model(""));

        imp.searchbar.connect_changed(clone!(
            #[strong]
            launcher,
            move |searchbar| {
                let query = searchbar.text().to_string();
                launcher.active_provider().map(|p| p.update_model(&query));
            }
        ));

        launcher
    }

    fn active_provider(&self) -> Option<Rc<dyn Provider>> {
        let name = self.active_provider_name()?;
        let providers = self.imp().providers.borrow();

        Some(providers.get(&name)?.clone())
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.get_visible();

        if target_visibility {
            self.present();
        } else {
            self.set_visible(target_visibility);
            self.imp().searchbar.set_text("");
            self.active_provider().map(|p| p.reset());
        }
    }
}
