pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;
pub use workspaces::Workspaces;

use crate::common::Metadata;
use grapes::Component;
use std::rc::Rc;

/// Panel module factory
pub trait ModuleFactory {
    type Config;
    type Component: Component;

    fn create(
        config: Rc<Self::Config>,
        meta: Metadata,
    ) -> anyhow::Result<Self::Component>;
}
