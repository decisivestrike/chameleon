use gtk::glib;
use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;

mod imp {
    use super::*;
    use gtk::prelude::OrientableExt;
    use gtk::subclass::prelude::*;
    use gtk::{Align, Orientation};
    use std::cell::Cell;
    use tracing::info;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::LauncherSearchbar)]
    pub struct LauncherSearchbarImp {
        #[property(get, set)]
        items_count: Cell<u64>,

        pub entry: gtk::Entry,
        pub counter: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherSearchbarImp {
        const NAME: &'static str = "ChameleonLauncherSearchbar";
        type Type = super::LauncherSearchbar;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for LauncherSearchbarImp {
        fn constructed(&self) {
            self.parent_constructed();

            let box_ = self.obj();
            box_.set_orientation(Orientation::Horizontal);
            box_.set_spacing(0);
            box_.set_hexpand(true);
            box_.set_valign(Align::Fill);
            box_.set_widget_name("searchbar");

            self.entry.set_hexpand(true);
            self.entry.set_valign(Align::Center);

            self.counter.set_hexpand(false);
            self.counter.set_valign(Align::Center);
            self.counter.set_widget_name("counter");

            box_.append(&self.entry);
            box_.append(&self.counter);

            box_.bind_property("items_count", &self.counter, "label")
                .transform_to(|_, count: u64| Some(count.to_string()))
                .sync_create()
                .build();

            box_.bind_property("items_count", &self.counter, "visible")
                .transform_to(|_, count: u64| Some(count > 0))
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for LauncherSearchbarImp {}

    impl BoxImpl for LauncherSearchbarImp {}
}

glib::wrapper! {
    pub struct LauncherSearchbar(ObjectSubclass<imp::LauncherSearchbarImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LauncherSearchbar {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn entry(&self) -> &gtk::Entry {
        &self.imp().entry
    }

    pub fn clear(&self) {
        self.imp().entry.set_text("");
    }

    pub fn query(&self) -> String {
        self.imp().entry.text().to_string()
    }
}

impl Default for LauncherSearchbar {
    fn default() -> Self {
        Self::new()
    }
}
