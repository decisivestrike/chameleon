pub mod clock;
use std::rc::Rc;

pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;
pub use workspaces::Workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

use crate::common::Metadata;
use grapes::Component;

/// Panel module factory
pub trait ModuleFactory {
    type Config;

    fn create(
        config: &Self::Config,
        meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>>;
}
