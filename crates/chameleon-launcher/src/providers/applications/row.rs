use crate::providers::ItemView;
use crate::providers::applications::entry::ApplicationEntry;
use gtk::glib;
use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use gtk::pango::{self};
    use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{Align, Orientation, glib};

    pub struct ApplicationRowImp {
        pub icon: gtk::Image,
        pub name: gtk::Label,
        pub comment: gtk::Label,
    }

    impl Default for ApplicationRowImp {
        fn default() -> Self {
            let icon = gtk::Image::builder()
                .icon_size(gtk::IconSize::Large)
                .pixel_size(48)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .build();

            let name = gtk::Label::builder()
                .xalign(0.0)
                .css_classes(["app-name"])
                .build();

            let comment = gtk::Label::builder()
                .single_line_mode(true)
                .max_width_chars(60)
                .ellipsize(pango::EllipsizeMode::End)
                .xalign(0.0)
                .css_classes(["app-comment"])
                .build();

            Self {
                icon,
                name,
                comment,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationRowImp {
        const NAME: &'static str = "LauncherAppCard";
        type Type = super::ApplicationRow;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ApplicationRowImp {
        fn constructed(&self) {
            self.parent_constructed();

            let text_container = gtk::Box::builder()
                .orientation(Orientation::Vertical)
                .halign(Align::Center)
                .valign(Align::Center)
                .spacing(4)
                .css_classes(["text-container"])
                .build();

            text_container.append(&self.name);
            text_container.append(&self.comment);

            let obj = self.obj();
            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_width_request(500); // later
            obj.set_orientation(Orientation::Horizontal);
            obj.set_spacing(12);
            obj.add_css_class("app");

            obj.append(&self.icon);
            obj.append(&text_container);
        }
    }

    impl WidgetImpl for ApplicationRowImp {}

    impl BoxImpl for ApplicationRowImp {}
}

glib::wrapper! {
    pub struct ApplicationRow(ObjectSubclass<imp::ApplicationRowImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ApplicationRow {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl ItemView for ApplicationRow {
    type Data = ApplicationEntry;

    fn bind(&self, entry: &Self::Data) {
        let imp = self.imp();

        imp.icon.set_icon_name(Some(&entry.icon()));
        imp.name.set_text(&entry.name());
        imp.comment.set_text(&entry.comment());
    }
}

impl Default for ApplicationRow {
    fn default() -> Self {
        Self::new()
    }
}
