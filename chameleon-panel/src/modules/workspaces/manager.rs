use crate::modules::workspaces::event::WorkspaceEvent;
use chameleon_core::errors::MonitorError;
use chameleon_ipc::hyprland::events::EVENTS;
use chameleon_ipc::hyprland::{HyprEvent, Workspace};
use grapes::RT;
use grapes::gtk::gdk;
use grapes::prelude::MonitorExt;
use grapes::tokio::select;
use grapes::tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio_util::sync::CancellationToken;

static WORKSPACES_MANAGER: LazyLock<RwLock<WorkspacesManager>> =
    LazyLock::new(|| Default::default());

static CANCELATION_TOKEN: RwLock<Option<CancellationToken>> =
    RwLock::const_new(None);

async fn send_event(monitor_connector: &String, event: WorkspaceEvent) {
    let workspaces_manager = WORKSPACES_MANAGER.read().await;

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

#[derive(Default)]
pub struct WorkspacesManager {
    instances: HashMap<String, mpsc::Sender<WorkspaceEvent>>,
}

/// Because hashmap contains gtk widgets. Но мы не обновляем эти виджеты в другом потоке
unsafe impl Send for WorkspacesManager {}
unsafe impl Sync for WorkspacesManager {}

impl WorkspacesManager {
    pub async fn register(
        monitor: &gdk::Monitor,
        sender: mpsc::Sender<WorkspaceEvent>,
    ) -> anyhow::Result<()> {
        if CANCELATION_TOKEN.read().await.is_none() {
            log::info!("Starting workspace event handler");

            let token = CancellationToken::new();
            RT.spawn(Self::event_handler(token.clone()));

            *CANCELATION_TOKEN.write().await = Some(token);
        }

        let monitor_connector = monitor
            .connector()
            .ok_or(MonitorError::NoConnector)?
            .to_string();

        WORKSPACES_MANAGER
            .write()
            .await
            .instances
            .insert(monitor_connector, sender);

        Ok(())
    }

    pub async fn event_handler(token: CancellationToken) -> anyhow::Result<()> {
        let mut events_receiver = EVENTS.subscribe();
        let mut active_workspace = Workspace::active().await.unwrap();

        loop {
            select! {
                maybe_event = events_receiver.recv() => {
                    let event = match maybe_event {
                        Ok(event) => event,
                        Err(e) => {
                            log::error!("{e}");
                            continue;
                        }
                    };

                    if let Err(e) = Self::handle_event(event, &mut active_workspace).await {
                        log::error!("{e}");
                    };
                }
                _ = token.cancelled() => {
                    log::info!("Shutting down workspace event handler...");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(
        event: HyprEvent,
        active_workspace: &mut Workspace,
    ) -> anyhow::Result<()> {
        match event {
            // Смена активного workspace
            HyprEvent::WorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Deactivate(active_workspace.id),
                )
                .await;

                active_workspace.id = id;

                send_event(
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
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Deactivate(active_workspace.id),
                )
                .await;

                active_workspace.id = workspace_id;
                active_workspace.monitor = monitor_connector;

                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Activate(active_workspace.id),
                )
                .await;
            }
            HyprEvent::CreateWorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Create(id),
                )
                .await;
            }
            HyprEvent::DestroyWorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Destroy(id),
                )
                .await;
            }
            _ => (),
        };

        Ok(())
    }
}

impl Drop for WorkspacesManager {
    fn drop(&mut self) {
        if let Some(token) = CANCELATION_TOKEN.blocking_write().take() {
            token.cancel();
        }

        println!("drop ws manager")
    }
}
