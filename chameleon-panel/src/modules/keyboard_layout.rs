use crate::{common::Metadata, modules::ModuleFactory};
use anyhow::Result;
use grapes::Component;
use grapes_components::StatefullLabel;
use std::rc::Rc;

#[derive(Debug, Component)]
pub struct KeyboardLayout {
    #[root]
    label: StatefullLabel<String>,
}

impl ModuleFactory for KeyboardLayout {
    type Config = ();

    fn create(
        config: &Self::Config,
        meta: &Metadata,
    ) -> Result<Rc<dyn Component>> {
        todo!()
    }
}

impl KeyboardLayout {}
