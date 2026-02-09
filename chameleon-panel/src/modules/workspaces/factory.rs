use crate::{
    common::Metadata,
    modules::{
        AsyncModuleFactory, Workspaces,
        workspaces::manager::{WorkspacesInstance, WorkspacesManager},
    },
};
use chameleon_config::panel::WorkspacesConfig;
use grapes::{Component, tokio::sync::mpsc};
use std::rc::Rc;

pub struct WorkspacesFactory;

impl AsyncModuleFactory for WorkspacesFactory {
    type Config = WorkspacesConfig;
    type Component = Workspaces;

    async fn create(
        config: Rc<WorkspacesConfig>,
        meta: Metadata,
    ) -> anyhow::Result<Workspaces> {
        let (sender, receiver) = mpsc::channel(64);

        let workspaces = Workspaces::new(config, meta.clone(), receiver);
        let widget = workspaces.as_widget_ref().clone();
        let instance = WorkspacesInstance::new(widget, sender);
        WorkspacesManager::register(&meta.monitor, instance);

        Ok(workspaces)
    }
}
