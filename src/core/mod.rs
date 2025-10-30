use grapes::{
    glib::object::Cast,
    gtk::gdk::{self, prelude::DisplayExt},
};

pub fn monitors() -> Vec<gdk::Monitor> {
    let display = gdk::Display::default().expect("No display");

    display
        .monitors()
        .into_iter()
        .filter_map(|obj| obj.ok()?.downcast::<gdk::Monitor>().ok())
        .collect()
}
