pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

// pub mod workspaces;
// pub use workspaces::Workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

// pub mod pulseaudio;
// pub use pulseaudio::Pulseaudio;

pub mod separator;

use gtk::Orientation;
use gtk::gdk::Monitor;
use std::rc::Rc;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum Error {
    #[error("unsupported device: {0}")]
    UnsupportedDevice(&'static str),
    #[error("unsupported compositor: {0}")]
    UnsupportedCompositor(&'static str),
}

pub trait PanelModule {
    type Rules;

    fn create(
        rules: Self::Rules,
        meta: Rc<Metadata>,
    ) -> Result<gtk::Widget, Error>;
}
