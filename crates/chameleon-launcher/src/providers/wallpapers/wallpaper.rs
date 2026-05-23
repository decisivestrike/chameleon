use gtk::glib::{self, Object};
use std::path::PathBuf;

mod imp {
    use super::*;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Wallpaper)]
    pub struct WallpaperImp {
        #[property(get, set)]
        path: RefCell<PathBuf>,

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
    pub fn new(path: PathBuf) -> Self {
        Object::builder().property("path", path).build()
    }
}
