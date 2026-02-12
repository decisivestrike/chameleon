use crate::modules::{KeyboardLayout, ModuleFactory};

pub(super) struct KeyboardLayoutFactory;

impl ModuleFactory for KeyboardLayoutFactory {
    type Config = ();

    type Module = KeyboardLayout;

    fn create(
        config: &std::rc::Rc<Self::Config>,
        meta: &crate::common::Metadata,
    ) -> Self::Module {
        todo!()
    }
}
