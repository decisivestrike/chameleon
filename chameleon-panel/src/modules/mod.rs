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

/// Panel module factory
pub trait ModuleFactory {
    type Config;
    type Module: Component;

    fn create(
        config: &Self::Config,
        meta: &Metadata,
    ) -> anyhow::Result<Self::Module>;

    fn boxed(
        config: &Self::Config,
        meta: &Metadata,
    ) -> anyhow::Result<Box<dyn Component>> {
        let instance = Self::create(config, meta)?;

        Ok(Box::new(instance))
    }
}
