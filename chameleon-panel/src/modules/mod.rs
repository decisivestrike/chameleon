pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;
pub use workspaces::Workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

use crate::common::Metadata;
use grapes::Component;
use std::rc::Rc;

/// Panel module factory
pub trait ModuleFactory {
    type Config;
    type Module: Component;

    fn create(config: &Rc<Self::Config>, meta: &Metadata) -> Self::Module;

    fn boxed(config: &Rc<Self::Config>, meta: &Metadata) -> Box<dyn Component> {
        let instance = Self::create(config, meta);

        Box::new(instance)
    }
}
