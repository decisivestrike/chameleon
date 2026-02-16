use crate::modules::workspaces::event::WorkspaceEvent;
use chameleon_core::errors::MonitorError;
use chameleon_ipc::hyprland::events::EVENT_SENDER;
use chameleon_ipc::hyprland::{HyprEvent, Workspace};
use dashmap::DashMap;
use grapes::RT;
use grapes::gtk::gdk;
use grapes::prelude::MonitorExt;
use grapes::tokio::sync::mpsc;
use std::sync::LazyLock;

static INSTANCES: LazyLock<DashMap<String, mpsc::Sender<WorkspaceEvent>>> =
    LazyLock::new(|| {
        RT.spawn(WorkspacesManager::event_handler());
        Default::default()
    });

pub(super) struct WorkspacesManager;

impl WorkspacesManager {
    pub async fn register(
        monitor: &gdk::Monitor,
        sender: mpsc::Sender<WorkspaceEvent>,
    ) -> anyhow::Result<()> {
        let monitor_connector = monitor
            .connector()
            .ok_or(MonitorError::NoConnector)?
            .to_string();

        INSTANCES.insert(monitor_connector, sender);

        Ok(())
    }

    async fn event_handler() -> ! {
        let mut events_receiver = EVENT_SENDER.subscribe();
        let mut active_workspace = Workspace::active().await.unwrap();

        loop {
            let maybe_event = events_receiver.recv().await;
            let event = match maybe_event {
                Ok(event) => event,
                Err(e) => {
                    log::error!("In event handler: {e}");
                    continue;
                }
            };

            Self::handle_event(event, &mut active_workspace).await;
        }
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
        let maybe_sender = INSTANCES.get(monitor_connector);

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
