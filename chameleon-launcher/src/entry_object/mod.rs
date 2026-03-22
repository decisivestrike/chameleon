mod imp;

use glib::Object;
use grapes::gtk::glib;

glib::wrapper! {
    pub struct EntryInfo(ObjectSubclass<imp::EntryObject>);
}

impl EntryInfo {
    pub fn new(
        name: String,
        exec: String,
        comment: String,
        icon: String,
        terminal: bool,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("exec", exec)
            .property("comment", comment)
            .property("icon", icon)
            .property("terminal", terminal)
            .build()
    }
}
