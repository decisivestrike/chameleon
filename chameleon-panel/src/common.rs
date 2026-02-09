use grapes::gtk::{Orientation, gdk::Monitor};

#[derive(Clone)]
pub struct Metadata {
    pub monitor: Monitor,
    pub orientation: Orientation,
}
