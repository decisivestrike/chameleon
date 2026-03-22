use glib::Properties;
use grapes::gtk::glib;
use grapes::gtk::prelude::*;
use grapes::gtk::subclass::prelude::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::EntryInfo)]
pub struct EntryObject {
    #[property(get, set)]
    name: Rc<RefCell<String>>,
    #[property(get, set)]
    exec: Rc<RefCell<String>>,
    #[property(get, set)]
    comment: Rc<RefCell<String>>,
    #[property(get, set)]
    icon: Rc<RefCell<String>>,
    #[property(get, set)]
    terminal: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for EntryObject {
    const NAME: &'static str = "ChameleonDesktopEntryObject";
    type Type = super::EntryInfo;
}

#[glib::derived_properties]
impl ObjectImpl for EntryObject {}
