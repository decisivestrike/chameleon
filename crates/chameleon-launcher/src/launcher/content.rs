use crate::config::LauncherConfig;
use crate::providers::Provider;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::{Align, PolicyType, ScrolledWindow, StackTransitionType, glib};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::info;

mod imp {
    use super::*;
    use crate::launcher::switcher::ProviderSwitcher;
    use crate::providers::Provider;
    use gtk::prelude::OrientableExt;
    use gtk::subclass::prelude::*;
    use gtk::{Align, Orientation};

    #[derive(Default)]
    pub struct LauncherContentImp {
        pub searchbar: gtk::Entry,
        pub switcher: ProviderSwitcher,
        pub stack: gtk::Stack,
        pub count: gtk::Label,
        pub providers: RefCell<HashMap<String, Rc<dyn Provider>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherContentImp {
        const NAME: &'static str = "ChameleonLauncherContent";
        type Type = super::LauncherContent;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for LauncherContentImp {
        fn constructed(&self) {
            self.parent_constructed();

            self.stack.set_hhomogeneous(false);
            self.stack.set_vhomogeneous(false);
            self.stack.set_interpolate_size(false);
            self.stack.set_transition_duration(0);
            self.stack.set_transition_type(StackTransitionType::None);

            self.switcher.set_stack(&self.stack);
            self.switcher.set_hexpand(false);
            self.switcher.set_halign(Align::Center);

            let box_ = self.obj();
            box_.set_orientation(Orientation::Vertical);
            box_.set_valign(Align::Fill);
            box_.set_halign(Align::Fill);
            box_.set_hexpand(true);
            box_.set_vexpand(true);
            box_.set_homogeneous(false);
            box_.set_spacing(0);
            box_.set_widget_name("launcher");

            box_.append(&self.searchbar);
            box_.append(&self.switcher);
            box_.append(&self.stack);
            // provider.append(&self.count);
        }
    }

    impl WidgetImpl for LauncherContentImp {}

    impl BoxImpl for LauncherContentImp {}
}

glib::wrapper! {
    pub struct LauncherContent(ObjectSubclass<imp::LauncherContentImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LauncherContent {
    pub fn new(config: LauncherConfig) -> Self {
        let content: Self = Object::builder().build();
        let imp = content.imp();

        imp.searchbar
            .set_placeholder_text(Some(&config.searchbar_placeholder));

        let providers = config.instantiate_providers();

        for provider in providers.into_iter() {
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
                .name(provider.name())
                .child(&provider.view())
                .build();

            let name = Self::capitalize(provider.name());
            imp.switcher.append_page(&name, &scrolled_window);
            imp.providers.borrow_mut().insert(name, provider);
        }

        imp.searchbar.connect_changed(clone!(
            #[strong]
            content,
            move |searchbar| {
                let query = searchbar.text();
                content.update_model(&query);
            }
        ));

        imp.stack.connect_visible_child_name_notify(clone!(
            #[strong]
            content,
            move |stack| {
                if let Some(name) = stack.visible_child_name() {
                    info!("Active provider: {}", name);
                }
                content.update_model_from_query();
            }
        ));

        content
    }

    pub fn update_model_from_query(&self) {
        let searchbar = &self.imp().searchbar;
        let query = searchbar.text();
        self.update_model(&query);
    }

    pub fn invoke_provider_action(&self) -> bool {
        if let Some(provider) = self.active_provider() {
            provider.invoke_action()
        } else {
            false
        }
    }

    pub fn reset_state(&self) {
        self.imp().searchbar.set_text("");
        self.active_provider().map(|p| p.reset_selection());
    }

    fn update_model(&self, query: &str) {
        if let Some(provider) = self.active_provider() {
            info!("Update: {}, {}", provider.name(), query);
            provider.update_model(&query);

            if provider.len() > 0 {
                let label = format!("{}", provider.len().to_string());
                self.imp().count.set_label(&label);
                self.imp().count.set_visible(true);
            } else {
                self.imp().count.set_visible(false);
            }
        }
    }

    fn active_provider(&self) -> Option<Rc<dyn Provider>> {
        let name = self.imp().stack.visible_child_name()?.to_string();
        let providers = self.imp().providers.borrow();

        Some(providers.get(&name)?.clone())
    }

    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();

        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>()
                    + chars.as_str().to_lowercase().as_str()
            }
        }
    }
}
