use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

#[derive(Properties, Default)]
#[properties(wrapper_type = super::ApplicationEntry)]
pub struct EntryObject {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    exec: RefCell<String>,
    #[property(get, set)]
    comment: RefCell<String>,
    #[property(get, set)]
    icon: RefCell<String>,
    #[property(get, set)]
    terminal: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for EntryObject {
    const NAME: &'static str = "ChameleonDesktopEntryObject";
    type Type = super::ApplicationEntry;
}

#[glib::derived_properties]
impl ObjectImpl for EntryObject {}
