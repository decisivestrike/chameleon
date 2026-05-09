use chameleon_ipc::CompositorVariant;
use gtk::Orientation;
use gtk::gdk::Monitor;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub monitor: Monitor,
    pub orientation: Orientation,
    pub compositor: &'static CompositorVariant,
}
