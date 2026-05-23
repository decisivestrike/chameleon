use gtk::gdk;
use gtk::glib::{self, Object};
use std::path::Path;

mod imp {
    use super::*;
    use gtk::gdk;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Wallpaper)]
    pub struct WallpaperImp {
        #[property(get, set)]
        texture: RefCell<Option<gdk::Texture>>,

        #[property(get, set)]
        picture_name: RefCell<String>,

        #[property(get, set, default = -1)]
        fuzzy_score: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperImp {
        const NAME: &'static str = "ChameleonLauncherWallpaperPath";
        type Type = super::Wallpaper;
    }

    #[glib::derived_properties]
    impl ObjectImpl for WallpaperImp {}
}

glib::wrapper! {
    pub struct Wallpaper(ObjectSubclass<imp::WallpaperImp>);
}

impl Wallpaper {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let wallpaper: Self = Object::builder().build();
        let os_filename = path.as_ref().file_name().unwrap();
        wallpaper.set_picture_name(os_filename.to_string_lossy());

        let texture = gdk::Texture::from_filename(&path).unwrap();
        wallpaper.set_texture(texture);

        wallpaper
    }
}
