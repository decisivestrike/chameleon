use crate::config::LauncherConfig;
use crate::providers::Provider;
use gtk::gdk::Key;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::{
    Align, EventControllerKey, PolicyType, PropagationPhase, ScrolledWindow,
    StackTransitionType, glib,
};
use layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::info;

mod imp {
    use super::*;
    use gtk::subclass::prelude::*;
    use gtk::{Orientation, glib};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Launcher)]
    pub struct LauncherImp {
        pub container: gtk::Box,

        pub searchbar: gtk::Entry,
        pub switcher: gtk::StackSwitcher,
        pub stack: gtk::Stack,
        pub separator: gtk::Separator,
        pub count: gtk::Label,
        pub providers: RefCell<HashMap<String, Rc<dyn Provider>>>,
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

            self.container.set_orientation(Orientation::Vertical);
            self.container.set_valign(Align::Fill);
            self.container.set_halign(Align::Fill);
            self.container.set_hexpand(true);
            self.container.set_vexpand(true);
            self.container.set_homogeneous(false);
            self.container.set_spacing(0);

            self.switcher.set_stack(Some(&self.stack));

            self.container.append(&self.searchbar);
            self.container.append(&self.switcher);
            self.container.append(&self.stack);
            self.container.set_widget_name("launcher");

            self.separator.set_orientation(Orientation::Horizontal);
            self.container.append(&self.separator);
            self.container.append(&self.count);

            self.stack.set_hhomogeneous(false);
            self.stack.set_vhomogeneous(false);

            let obj = self.obj();
            obj.init_layer_shell();
            obj.set_default_size(-1, -1); // Auto size
            obj.set_vexpand(false);
            obj.set_hexpand(false);
            obj.set_namespace(Some("chameleon-launcher"));
            obj.set_widget_name("launcher-window");

            // TODO: Make configurable
            obj.set_layer(Layer::Top);
            obj.set_keyboard_mode(if true {
                KeyboardMode::Exclusive
            } else {
                KeyboardMode::OnDemand
            });

            obj.set_child(Some(&self.container));
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

        let imp = launcher.imp();
        imp.stack.set_interpolate_size(false);
        imp.stack.set_transition_duration(0);
        imp.stack.set_transition_type(StackTransitionType::None);

        imp.searchbar
            .set_placeholder_text(Some(&config.searchbar_placeholder));

        let providers = config.instantiate_providers();

        fn capitalize_unicode(s: &str) -> String {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + chars.as_str().to_lowercase().as_str()
                }
            }
        }

        for p in providers.into_iter().rev() {
            let scrolled_window = ScrolledWindow::builder()
                .propagate_natural_height(true)
                .halign(Align::Fill)
                .valign(Align::Fill)
                .hscrollbar_policy(PolicyType::Never)
                .vscrollbar_policy(PolicyType::Automatic)
                .can_focus(false)
                .can_target(true)
                .min_content_width(480)
                .max_content_width(1200)
                .min_content_height(0)
                .max_content_height(420)
                .hexpand(false)
                .vexpand(false)
                .overlay_scrolling(true)
                .name(p.name())
                .child(&p.view())
                .build();

            launcher.imp().stack.add_titled(
                &scrolled_window,
                Some(p.name()),
                &capitalize_unicode(p.name()),
            );

            launcher
                .imp()
                .providers
                .borrow_mut()
                .insert(p.name().to_string(), p);
        }

        // Config
        // imp.stack.set_visible_child_name("applications");

        let root_controller = EventControllerKey::new();
        root_controller.connect_key_pressed(clone!(
            #[strong]
            launcher,
            move |_, key, _, _| {
                match key {
                    Key::Left => glib::Propagation::Stop,
                    Key::Down => {
                        if let Some(provider) = launcher.active_provider() {
                            provider.select_below();
                        }
                        glib::Propagation::Stop
                    }
                    Key::Tab => {
                        // let container = &launcher.imp().container;
                        // container.focus_child().map(|focused| {
                        //     container.set_focus_child(focused.next_sibling())
                        // });

                        glib::Propagation::Stop
                    }
                    Key::Return => {
                        launcher
                            .active_provider()
                            .map(|p| p.invoke_action())
                            .map(|invoked| {
                                if invoked {
                                    launcher.toggle_visibility()
                                }
                            });
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

        imp.searchbar.connect_changed(clone!(
            #[strong]
            launcher,
            move |searchbar| {
                let query = searchbar.text();
                launcher.update_model(&query);
            }
        ));

        imp.stack.connect_visible_child_name_notify(clone!(
            #[strong]
            launcher,
            move |stack| {
                if let Some(name) = stack.visible_child_name() {
                    info!("Active provider: {}", name);
                }
                let searchbar = &launcher.imp().searchbar;
                let query = searchbar.text();
                launcher.update_model(&query);
            }
        ));

        launcher
    }

    fn update_model(&self, query: &str) {
        if let Some(provider) = self.active_provider() {
            info!("Update: {}, {}", provider.name(), query);
            provider.update_model(&query);

            if provider.len() > 0 {
                let label = format!("{}", provider.len().to_string());
                self.imp().count.set_label(&label);
                self.imp().separator.set_visible(true);
                self.imp().count.set_visible(true);
            } else {
                self.imp().separator.set_visible(false);
                self.imp().count.set_visible(false);
            }
        }
    }

    fn active_provider(&self) -> Option<Rc<dyn Provider>> {
        let name = self.imp().stack.visible_child_name()?.to_string();
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
