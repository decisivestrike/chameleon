use gtk::gdk::prelude::DisplayExt;
use gtk::gdk::{self};
use gtk::glib::object::Cast;

pub trait GtkeMonitorExt {
    fn each() -> Vec<gdk::Monitor>;
}

impl GtkeMonitorExt for gdk::Monitor {
    fn each() -> Vec<gdk::Monitor> {
        let display = gdk::Display::default().expect("No display");

        display
            .monitors()
            .into_iter()
            .filter_map(|obj| obj.ok()?.downcast::<gdk::Monitor>().ok())
            .collect()
    }
}
