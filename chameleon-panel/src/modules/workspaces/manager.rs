use crate::modules::workspaces::event::WorkspaceEvent;
use chameleon_core::errors::MonitorError;
use chameleon_ipc::hyprland::events::EVENTS;
use chameleon_ipc::hyprland::{HyprEvent, Workspace};
use grapes::RT;
use grapes::gtk::gdk;
use grapes::prelude::MonitorExt;
use grapes::tokio::select;
use grapes::tokio::sync::{
    RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard, mpsc,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub static WORKSPACES_MANAGER: RwLock<Option<WorkspacesManager>> =
    RwLock::const_new(None);

#[derive(Default)]
pub struct WorkspacesManager {
    pub(super) instances: HashMap<String, mpsc::Sender<WorkspaceEvent>>,
    token: Option<CancellationToken>,
}

impl WorkspacesManager {
    pub async fn register(
        monitor: &gdk::Monitor,
        sender: mpsc::Sender<WorkspaceEvent>,
    ) -> anyhow::Result<()> {
        Self::init_if_none().await;

        let monitor_connector = monitor
            .connector()
            .ok_or(MonitorError::NoConnector)?
            .to_string();

        WorkspacesManager::write()
            .await
            .instances
            .insert(monitor_connector, sender);

        Ok(())
    }

    pub async fn unregister(monitor_connector: Arc<String>) {
        WorkspacesManager::write()
            .await
            .instances
            .remove(&*monitor_connector);

        if WorkspacesManager::read().await.instances.len() == 0 {
            WORKSPACES_MANAGER.write().await.take();
        }
    }

    /// Initializes singleton if it none
    async fn init_if_none() {
        if WORKSPACES_MANAGER.read().await.is_none() {
            let mut ws_manager = WORKSPACES_MANAGER.write().await;
            *ws_manager = Some(WorkspacesManager::default());

            let token = CancellationToken::new();
            RT.spawn(Self::event_handler(token.clone()));

            if let Some(manager) = ws_manager.as_mut() {
                manager.token = Some(token);
            }
        }
    }

    pub(super) async fn read() -> RwLockReadGuard<'static, WorkspacesManager> {
        let guard = WORKSPACES_MANAGER.read().await;

        RwLockReadGuard::map(guard, |g| g.as_ref().unwrap())
    }

    pub(super) async fn write()
    -> RwLockMappedWriteGuard<'static, WorkspacesManager> {
        let guard = WORKSPACES_MANAGER.write().await;

        RwLockWriteGuard::map(guard, |g| g.as_mut().unwrap())
    }

    async fn event_handler(token: CancellationToken) -> anyhow::Result<()> {
        let mut events_receiver = EVENTS.subscribe();
        let mut active_workspace = Workspace::active().await?;

        loop {
            select! {
                maybe_event = events_receiver.recv() => {
                    let event = match maybe_event {
                        Ok(event) => event,
                        Err(e) => {
                            log::error!("In event handler: {e}");
                            continue;
                        }
                    };

                    Self::handle_event(event, &mut active_workspace).await;
                }
                _ = token.cancelled() => { break; }
            }
        }

        log::debug!("Workspace event handler was stopped");

        Ok(())
    }

    async fn handle_event(event: HyprEvent, active_workspace: &mut Workspace) {
        match event {
            // Смена активного workspace
            HyprEvent::WorkspaceV2 { id, name: _ } => {
                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Deactivate(active_workspace.id),
                )
                .await;

                active_workspace.id = id;

                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Activate(active_workspace.id),
                )
                .await;
            }
            // Смена активного монитора
            HyprEvent::FocusedMonV2 {
                monitor_connector,
                workspace_id,
            } => {
                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Deactivate(active_workspace.id),
                )
                .await;

                active_workspace.id = workspace_id;
                active_workspace.monitor = monitor_connector;

                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Activate(active_workspace.id),
                )
                .await;
            }
            HyprEvent::CreateWorkspaceV2 { id, name: _ } => {
                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Create(id),
                )
                .await;
            }
            HyprEvent::DestroyWorkspaceV2 { id, name: _ } => {
                Self::send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Destroy(id),
                )
                .await;
            }
            _ => (),
        };
    }

    async fn send_event(monitor_connector: &String, event: WorkspaceEvent) {
        let workspaces_manager = WorkspacesManager::read().await;

        let maybe_sender = workspaces_manager
            .instances
            .iter()
            .find(|(connector, _)| *connector == monitor_connector)
            .map(|(_, sender)| sender);

        match maybe_sender {
            Some(sender) => {
                if let Err(e) = sender.send(event).await {
                    log::error!("Error while sending event: {e}");
                }
            }
            None => log::warn!("Cant find channel for '{}'", monitor_connector),
        }
    }
}

impl Drop for WorkspacesManager {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            token.cancel();
        }

        log::debug!("Workspaces manager was dropped")
    }
}
