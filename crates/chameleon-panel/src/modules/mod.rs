pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

pub mod pulseaudio;
pub use pulseaudio::Pulseaudio;

pub mod launcher_toggle;
pub mod mpris;
pub mod separator;
pub mod tray;

use anyhow::Result;
use gtk::gdk::Monitor;
use gtk::{Orientation, Widget};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub monitor: Monitor,
    pub orientation: Orientation,
}

impl Metadata {
    pub fn new_rc(monitor: Monitor, orientation: Orientation) -> Rc<Self> {
        Self {
            monitor,
            orientation,
        }
        .into()
    }
}

pub trait PanelModule {
    type Rules;

    fn create(rules: Self::Rules, meta: Rc<Metadata>) -> Result<Widget>;
}
