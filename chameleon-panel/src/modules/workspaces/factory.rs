use crate::{
    common::Metadata,
    modules::{
        ModuleFactory, Workspaces, workspaces::manager::WorkspacesManager,
    },
};
use chameleon_config::panel::WorkspacesConfig;
use grapes::{RT, tokio::sync::mpsc};
use std::rc::Rc;

pub struct WorkspacesFactory;

impl ModuleFactory for WorkspacesFactory {
    type Config = WorkspacesConfig;
    type Component = Workspaces;

    /// Creates `Workspaces` instance and register it in `WorkspacesManager`
    fn create(
        config: &Rc<WorkspacesConfig>,
        meta: &Metadata,
    ) -> anyhow::Result<Workspaces> {
        let (sender, receiver) = mpsc::channel(64);

        let workspaces = Workspaces::new(&config, &meta, receiver);

        if let Err(e) =
            RT.block_on(WorkspacesManager::register(&meta.monitor, sender))
        {
            log::error!("{e}");
        };

        Ok(workspaces)
    }
}
