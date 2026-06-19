use crate::config::LauncherConfig;
use crate::launcher::LauncherSearchbar;
use crate::providers::{Direction, Provider};
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::{
    Align, PolicyType, ScrolledWindow, StackTransitionType, Widget, glib,
};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::info;

const DEFAULT_PAGE_NAME: &str = "default";
const NOTFOUND_PAGE_NAME: &str = "notfound";

mod imp {
    use super::*;
    use crate::launcher::LauncherSearchbar;
    use crate::launcher::switcher::ProviderSwitcher;
    use crate::providers::Provider;
    use gtk::prelude::OrientableExt;
    use gtk::subclass::prelude::*;
    use gtk::{Align, Orientation};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::LauncherContent)]
    pub struct LauncherContentImp {
        pub searchbar: LauncherSearchbar,
        pub switcher: ProviderSwitcher,
        pub inner_stack: gtk::Stack,
        pub outer_stack: gtk::Stack,
        pub providers: RefCell<Vec<Rc<dyn Provider>>>,
    }

    impl LauncherContentImp {
        fn setup_stack(stack: &gtk::Stack) {
            stack.set_hhomogeneous(false);
            stack.set_vhomogeneous(false);
            stack.set_interpolate_size(false);
            stack.set_transition_duration(0);
            stack.set_transition_type(StackTransitionType::None);
        }

        fn create_notfound_page() -> gtk::Box {
            let header = gtk::Label::builder()
                .label("Nothing found")
                .name("notfound-header")
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let comment = gtk::Label::builder()
                        .label("Well, this is awkward. Tell me you didn't just type random keyboard spam")
                        .name("notfound-comment")
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .build();

            let container = gtk::Box::new(Orientation::Vertical, 0);
            container.set_widget_name(NOTFOUND_PAGE_NAME);
            container.append(&header);
            container.append(&comment);

            container
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherContentImp {
        const NAME: &'static str = "ChameleonLauncherContent";
        type Type = super::LauncherContent;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for LauncherContentImp {
        fn constructed(&self) {
            self.parent_constructed();

            Self::setup_stack(&self.inner_stack);
            Self::setup_stack(&self.outer_stack);
            self.outer_stack
                .add_named(&self.inner_stack, Some(DEFAULT_PAGE_NAME));
            self.outer_stack.add_named(
                &Self::create_notfound_page(),
                Some(NOTFOUND_PAGE_NAME),
            );

            self.switcher.set_stack(&self.inner_stack);
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
            box_.append(&self.outer_stack);
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
        let searchbar = content.searchbar();
        let switcher = &imp.switcher;

        searchbar
            .entry()
            .set_placeholder_text(Some(&config.searchbar_placeholder));

        let providers = config.instantiate_providers();

        for (i, provider) in providers.into_iter().enumerate() {
            let page = Self::create_page(provider.name(), &provider.view());

            if i == 0 {
                searchbar.set_items_count(provider.len() as u32);
            }

            switcher.append_page(&provider.name(), &page);
            imp.providers.borrow_mut().push(provider);
        }

        searchbar.entry().connect_changed(clone!(
            #[strong]
            content,
            move |searchbar| {
                let query = searchbar.text();
                content.update_model(&query, false);
            }
        ));

        switcher.connect_active_index_notify(clone!(
            #[strong]
            content,
            move |_| {
                let provider = content.active_provider();
                info!("Active provider: {}", provider.name());
                content.update_model_from_query(true);
            }
        ));

        content
    }

    pub fn update_model_from_query(&self, provider_changed: bool) {
        let searchbar = self.searchbar();
        let query = searchbar.query();
        self.update_model(&query, provider_changed);
    }

    pub fn invoke_provider_action(&self) -> bool {
        let provider = self.active_provider();
        provider.invoke_action()
    }

    pub fn move_selection(&self, direction: Direction) {
        let provider = self.active_provider();
        provider.move_selection(direction);
    }

    pub fn reset_state(&self) {
        self.imp().searchbar.clear();
        self.active_provider().reset_selection();
    }

    fn create_page(name: &str, child: &impl IsA<Widget>) -> ScrolledWindow {
        ScrolledWindow::builder()
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
            .name(name)
            .child(child)
            .build()
    }

    fn update_model(&self, query: &str, provider_changed: bool) {
        let provider = self.active_provider();
        let outer_stack = self.outer_stack();

        info!("Update {}: '{}'", provider.name(), query);
        provider.update_model(&query, provider_changed);

        let count = provider.len();
        self.searchbar().set_items_count(count as u32);

        if count == 0 {
            outer_stack.set_visible_child_name(NOTFOUND_PAGE_NAME);
        } else {
            outer_stack.set_visible_child_name(DEFAULT_PAGE_NAME);
        }
    }

    fn searchbar(&self) -> &LauncherSearchbar {
        &self.imp().searchbar
    }

    fn outer_stack(&self) -> &gtk::Stack {
        &self.imp().outer_stack
    }

    fn active_provider(&self) -> Rc<dyn Provider> {
        let imp = self.imp();
        let i = imp.switcher.active_index();
        imp.providers.borrow()[i as usize].clone()
    }
}
