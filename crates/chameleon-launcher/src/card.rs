use crate::entry_object::ApplicationEntry;
use gtk::glib;
use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use gtk::pango::{self, EllipsizeMode, WrapMode};
    use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{Orientation, glib};

    pub struct CardImp {
        pub icon: gtk::Image,
        pub name: gtk::Label,
        pub comment: gtk::Label,
    }

    impl Default for CardImp {
        fn default() -> Self {
            let icon = gtk::Image::builder()
                .icon_size(gtk::IconSize::Large)
                .pixel_size(48)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .build();

            let name = gtk::Label::builder()
                .xalign(0.0)
                .css_classes(["name"])
                .build();

            let comment = gtk::Label::builder()
                .single_line_mode(true)
                .max_width_chars(60)
                .ellipsize(pango::EllipsizeMode::End)
                .xalign(0.0)
                .css_classes(["comment"])
                .build();

            Self {
                icon,
                name,
                comment,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardImp {
        const NAME: &'static str = "LauncherAppCard";
        type Type = super::Card;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for CardImp {
        fn constructed(&self) {
            self.parent_constructed();

            let text_container = gtk::Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(4)
                .css_classes(["text-container"])
                .build();

            text_container.append(&self.name);
            text_container.append(&self.comment);

            let obj = self.obj();
            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_width_request(600); // later
            obj.set_orientation(Orientation::Horizontal);
            obj.set_spacing(12);
            obj.add_css_class("app");

            obj.append(&self.icon);
            obj.append(&text_container);
        }
    }

    impl WidgetImpl for CardImp {}

    impl BoxImpl for CardImp {}
}

glib::wrapper! {
    pub struct Card(ObjectSubclass<imp::CardImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Card {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_data(&self, entry: &ApplicationEntry) {
        let imp = self.imp();

        imp.icon.set_icon_name(Some(&entry.icon()));
        imp.name.set_text(&entry.name());

        imp.comment.set_text(&entry.comment());
    }
}
