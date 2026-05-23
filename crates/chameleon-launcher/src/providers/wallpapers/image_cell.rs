use crate::providers::Bindable;
use crate::providers::wallpapers::wallpaper::Wallpaper;
use gtk::glib;
use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use gtk::pango::{self};
    use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{ContentFit, Orientation, glib};

    pub struct ImageCellImp {
        pub picture: gtk::Picture,
        pub name: gtk::Label,
    }

    impl Default for ImageCellImp {
        fn default() -> Self {
            let picture = gtk::Picture::builder()
                .content_fit(ContentFit::Contain)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .height_request(128)
                .build();

            let name = gtk::Label::builder()
                .single_line_mode(true)
                .max_width_chars(30)
                .ellipsize(pango::EllipsizeMode::End)
                .xalign(0.0)
                .css_classes(["name"])
                .build();

            Self { picture, name }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageCellImp {
        const NAME: &'static str = "LauncherWallpapersImageCell";
        type Type = super::ImageCell;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ImageCellImp {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_orientation(Orientation::Vertical);
            obj.set_spacing(0);
            obj.add_css_class("image-cell");

            obj.append(&self.picture);
            obj.append(&self.name);
        }
    }

    impl WidgetImpl for ImageCellImp {}

    impl BoxImpl for ImageCellImp {}
}

glib::wrapper! {
    pub struct ImageCell(ObjectSubclass<imp::ImageCellImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ImageCell {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Bindable for ImageCell {
    type Data = Wallpaper;

    fn bind_data(&self, wallpaper: &Self::Data) {
        let imp = self.imp();

        imp.picture.set_paintable(wallpaper.texture().as_ref());
        imp.name.set_text(&wallpaper.picture_name());
    }
}

impl Default for ImageCell {
    fn default() -> Self {
        Self::new()
    }
}
