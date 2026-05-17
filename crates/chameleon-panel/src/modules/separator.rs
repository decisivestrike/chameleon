use crate::modules::{Metadata, PanelModule};
use gtk::glib::object::Cast;
use std::rc::Rc;

/// Default gtk4 separator
pub struct Separator;

impl PanelModule for Separator {
    type Rules = ();

    fn create(_: (), meta: Rc<Metadata>) -> Result<gtk::Widget, super::Error> {
        Ok(gtk::Separator::new(meta.orientation).upcast())
    }
}
