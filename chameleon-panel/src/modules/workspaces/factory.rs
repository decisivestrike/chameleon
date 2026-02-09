use crate::{
    common::Metadata,
    modules::{
        ModuleFactory, Workspaces,
        workspaces::manager::{WorkspacesInstanceData, WorkspacesManager},
    },
};
use chameleon_config::panel::WorkspacesConfig;
use grapes::{Component, RT, tokio::sync::mpsc};
use std::rc::Rc;

pub struct WorkspacesFactory;

impl ModuleFactory for WorkspacesFactory {
    type Config = WorkspacesConfig;
    type Component = Workspaces;

    /// Creates `Workspaces` instance and register it in `WorkspacesManager`
    fn create(
        config: Rc<WorkspacesConfig>,
        meta: Metadata,
    ) -> anyhow::Result<Workspaces> {
        let (sender, receiver) = mpsc::channel(64);

        let workspaces = Workspaces::new(config, meta.clone(), receiver);

        let widget = workspaces.as_widget_ref().clone();
        let instance_data = WorkspacesInstanceData::new(widget, sender);

        if let Err(e) = RT
            .block_on(WorkspacesManager::register(&meta.monitor, instance_data))
        {
            log::error!("{e}");
        };

        Ok(workspaces)
    }
}
