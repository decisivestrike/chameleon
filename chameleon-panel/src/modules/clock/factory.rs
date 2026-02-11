use crate::{
    common::Metadata,
    modules::{Clock, ModuleFactory},
};
use chameleon_config::panel::ClockConfig;
use std::rc::Rc;

pub struct ClockFactory;

impl ModuleFactory for ClockFactory {
    type Config = ClockConfig;
    type Module = Clock;

    fn create(config: &Rc<Self::Config>, _meta: &Metadata) -> Self::Module {
        Clock::new(config)
    }
}
