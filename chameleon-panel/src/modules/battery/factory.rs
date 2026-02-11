use chameleon_config::panel::BatteryConfig;
use std::rc::Rc;

use crate::{
    common::Metadata,
    modules::{Battery, ModuleFactory},
};

pub struct BatteryFactory;

impl ModuleFactory for BatteryFactory {
    type Config = BatteryConfig;
    type Module = Battery;

    fn create(config: &Rc<Self::Config>, _meta: &Metadata) -> Self::Module {
        Battery::new(config)
    }
}
