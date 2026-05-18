use crate::modules::{Metadata, PanelModule};
use gtk::glib::object::Cast;
use std::rc::Rc;

/// Default gtk4 separator
pub struct Separator;

impl PanelModule for Separator {
    type Rules = ();

    fn create(_: (), meta: Rc<Metadata>) -> Result<gtk::Widget, super::Error> {
        let separator = gtk::Separator::builder()
            .orientation(meta.orientation)
            .build();

        Ok(separator.upcast())
    }
}
