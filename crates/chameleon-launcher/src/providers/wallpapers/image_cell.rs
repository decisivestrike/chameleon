use crate::providers::Bindable;
use crate::providers::wallpapers::wallpaper::Wallpaper;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object, clone};

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
                .content_fit(ContentFit::Cover)
                .width_request(160)
                .height_request(90)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .name("wallpaper-picture")
                .build();

            let name = gtk::Label::builder()
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .single_line_mode(true)
                .max_width_chars(30)
                .ellipsize(pango::EllipsizeMode::End)
                .name("wallpaper-name")
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
            obj.set_widget_name("wallpaper");

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

        if let Some(paintable) = wallpaper.texture().as_ref() {
            imp.picture.set_paintable(Some(paintable));
        } else {
            glib::timeout_add_seconds_local(
                1,
                clone!(
                    #[strong(rename_to=cell)]
                    self,
                    #[strong]
                    wallpaper,
                    move || {
                        if let Some(paintable) = wallpaper.texture().as_ref() {
                            cell.imp().picture.set_paintable(Some(paintable));

                            glib::ControlFlow::Break
                        } else {
                            glib::ControlFlow::Continue
                        }
                    }
                ),
            );
        }

        imp.name.set_text(&wallpaper.picture_name());
    }
}

impl Default for ImageCell {
    fn default() -> Self {
        Self::new()
    }
}
