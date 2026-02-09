use crate::modules::workspaces::event::WorkspaceEvent;
use chameleon_core::errors::MonitorError;
use chameleon_ipc::hyprland::events::EVENTS;
use chameleon_ipc::hyprland::{HyprEvent, Workspace};
use grapes::gtk::gdk;
use grapes::prelude::MonitorExt;
use grapes::tokio::sync::{
    RwLock, RwLockMappedWriteGuard, RwLockWriteGuard, mpsc,
};
use grapes::tokio::task::JoinHandle;
use grapes::{RT, gtk};
use std::collections::HashMap;
use std::sync::LazyLock;

static WORKSPACES_MANAGER: RwLock<Option<WorkspacesManager>> =
    RwLock::const_new(None);

static TASK_HANDLE: RwLock<Option<JoinHandle<()>>> = RwLock::const_new(None);

static ACTIVE_WORKSPACE: LazyLock<RwLock<Workspace>> = LazyLock::new(|| {
    let active_workspace = RT.block_on(Workspace::active());
    RwLock::new(active_workspace.unwrap())
});

pub struct WorkspacesInstance {
    pub widget: gtk::Widget,
    pub sender: mpsc::Sender<WorkspaceEvent>,
}

impl WorkspacesInstance {
    pub fn new(
        widget: gtk::Widget,
        sender: mpsc::Sender<WorkspaceEvent>,
    ) -> Self {
        Self { widget, sender }
    }
}

#[derive(Default)]
pub struct WorkspacesManager {
    instances: HashMap<String, WorkspacesInstance>,
}

unsafe impl Send for WorkspacesManager {}
unsafe impl Sync for WorkspacesManager {}

impl WorkspacesManager {
    pub async fn register(
        monitor: &gdk::Monitor,
        instance: WorkspacesInstance,
    ) -> anyhow::Result<()> {
        let mut workspaces_manager = WorkspacesManager::write().await?;

        let monitor_connector = monitor
            .connector()
            .ok_or(MonitorError::NoConnector)?
            .to_string();

        workspaces_manager
            .instances
            .insert(monitor_connector, instance);

        Ok(())
    }

    async fn init() -> anyhow::Result<()> {
        let mut workspaces_model = WORKSPACES_MANAGER.write().await;
        let mut task_handle = TASK_HANDLE.write().await;

        if workspaces_model.is_some() || task_handle.is_some() {
            panic!("Workspace model already initialized")
        }

        *workspaces_model = Some(WorkspacesManager::default());
        drop(workspaces_model);

        let join_handle = RT.spawn(WorkspacesManager::event_handler());
        *task_handle = Some(join_handle);

        Ok(())
    }

    async fn write()
    -> anyhow::Result<RwLockMappedWriteGuard<'static, WorkspacesManager>> {
        let workspaces_manager = WORKSPACES_MANAGER.read().await;

        if workspaces_manager.is_none() {
            drop(workspaces_manager);
            WorkspacesManager::init().await?;
        }

        let guard_with_option = WORKSPACES_MANAGER.write().await;
        let guard = RwLockWriteGuard::map(guard_with_option, |wm| {
            wm.as_mut().expect("already checked")
        });

        Ok(guard)
    }

    async fn send_event(
        &self,
        monitor: &gdk::Monitor,
        event: WorkspaceEvent,
    ) -> anyhow::Result<()> {
        let monitor_connector = monitor
            .connector()
            .ok_or(MonitorError::NoConnector)?
            .to_string();

        let maybe_instance = self
            .instances
            .iter()
            .find(|(connector, _)| *connector == &monitor_connector)
            .map(|(_, instance)| instance);

        match maybe_instance {
            Some(instance) => {
                if let Err(e) = instance.sender.send(event).await {
                    log::error!("{e}");
                }
            }
            None => log::warn!("Cant find channel for '{}'", monitor_connector),
        }

        Ok(())
    }

    async fn event_handler() {
        let mut events_receiver = EVENTS.subscribe();

        loop {
            let maybe_event = events_receiver.recv().await;

            let event = match maybe_event {
                Ok(event) => event,
                Err(e) => {
                    log::error!("{e}");
                    continue;
                }
            };

            match event {
                HyprEvent::WorkspaceV2 { id, name: _ } => {
                    send_to_monitor(
                        &self.active_workspace.monitor,
                        WorkspaceEvent::ChangeActive {
                            from: self.active_workspace.id,
                            to: id,
                        },
                    )
                    .await;

                    self.active_workspace.id = id;
                }
                HyprEvent::FocusedMonV2 {
                    monitor_name,
                    workspace_id,
                } => {
                    // Тут должны быть разные мониторы
                    send_to_monitor(
                        &monitor_name,
                        WorkspaceEvent::ChangeActive {
                            from: self.active_workspace.id,
                            to: workspace_id,
                        },
                    )
                    .await;

                    self.active_workspace.id = workspace_id;
                    self.active_workspace.monitor = monitor_name;
                }
                HyprEvent::CreateWorkspaceV2 { id, name: _ } => {
                    send_to_monitor(
                        &self.active_workspace.monitor,
                        WorkspaceEvent::Create(id),
                    )
                    .await;
                }
                HyprEvent::DestroyWorkspaceV2 { id, name: _ } => {
                    send_to_monitor(
                        &self.active_workspace.monitor,
                        WorkspaceEvent::Destroy(id),
                    )
                    .await;
                }
                _ => (),
            };
        }
    }
}
