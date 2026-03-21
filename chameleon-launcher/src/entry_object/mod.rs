mod imp;

use glib::Object;
use grapes::gtk::glib;

glib::wrapper! {
    pub struct EntryInfo(ObjectSubclass<imp::IntegerObject>);
}

impl EntryInfo {
    pub fn new(
        name: String,
        exec: String,
        comment: String,
        icon: String,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("exec", exec)
            .property("comment", comment)
            .property("icon", icon)
            .build()
    }
}
