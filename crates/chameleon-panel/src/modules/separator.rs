use crate::modules::{Metadata, PanelModule};
use anyhow::Result;
use gtk::Orientation;
use gtk::glib::object::Cast;
use std::rc::Rc;

/// Default gtk4 separator
pub struct Separator;

impl PanelModule for Separator {
    type Rules = ();

    fn create(_: (), meta: Rc<Metadata>) -> Result<gtk::Widget> {
        let orientation = match meta.orientation {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
            _ => unreachable!(),
        };

        let separator = gtk::Separator::new(orientation);
        Ok(separator.upcast())
    }
}
