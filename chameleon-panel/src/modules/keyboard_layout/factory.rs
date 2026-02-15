use crate::{
    common::Metadata,
    modules::{KeyboardLayout, ModuleFactory},
};

pub(super) struct KeyboardLayoutFactory;

impl ModuleFactory for KeyboardLayoutFactory {
    type Config = ();
    type Module = KeyboardLayout;

    fn create(config: &Self::Config, meta: &Metadata) -> Self::Module {
        todo!()
    }
}
