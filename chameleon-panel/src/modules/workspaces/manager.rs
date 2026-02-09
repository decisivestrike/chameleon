use crate::modules::workspaces::event::WorkspaceEvent;
use chameleon_core::errors::MonitorError;
use chameleon_ipc::hyprland::events::EVENTS;
use chameleon_ipc::hyprland::{HyprEvent, Workspace};
use grapes::glib::{ControlFlow, clone};
use grapes::gtk::gdk;
use grapes::prelude::{BoxExt, Cast, MonitorExt, WidgetExt};
use grapes::tokio::select;
use grapes::tokio::sync::{
    Mutex, RwLock, RwLockMappedWriteGuard, RwLockWriteGuard, mpsc,
};
use grapes::{RT, glib, gtk};
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio_util::sync::CancellationToken;

static WORKSPACES_MANAGER: RwLock<Option<WorkspacesManager>> =
    RwLock::const_new(None);

static CANCELATION_TOKEN: RwLock<Option<CancellationToken>> =
    RwLock::const_new(None);

static ACTIVE_WORKSPACE: LazyLock<Mutex<Workspace>> = LazyLock::new(|| {
    let active_workspace = RT.block_on(Workspace::active()).unwrap();
    Mutex::new(active_workspace)
});

async fn send_event(
    monitor_connector: &String,
    event: WorkspaceEvent,
) -> anyhow::Result<()> {
    let workspaces_manager = WorkspacesManager::write().await?;

    let maybe_sender = workspaces_manager
        .instances
        .iter()
        .find(|(connector, _)| *connector == monitor_connector)
        .map(|(_, instance)| instance.sender.clone());

    drop(workspaces_manager);

    match maybe_sender {
        Some(sender) => {
            if let Err(e) = sender.send(event).await {
                log::error!("{e}");
            }
        }
        None => log::warn!("Cant find channel for '{}'", monitor_connector),
    }

    Ok(())
}

pub struct WorkspacesInstanceData {
    pub widget: gtk::Widget,
    pub sender: mpsc::Sender<WorkspaceEvent>,
}

impl WorkspacesInstanceData {
    pub fn new(
        widget: gtk::Widget,
        sender: mpsc::Sender<WorkspaceEvent>,
    ) -> Self {
        Self { widget, sender }
    }
}

#[derive(Default)]
pub struct WorkspacesManager {
    instances: HashMap<String, WorkspacesInstanceData>,
}

/// Because hashmap contains gtk widgets. Но мы не обновляем эти виджеты в другом потоке
unsafe impl Send for WorkspacesManager {}
unsafe impl Sync for WorkspacesManager {}

impl WorkspacesManager {
    pub async fn register(
        monitor: &gdk::Monitor,
        instance: WorkspacesInstanceData,
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
        let mut workspaces_manager = WORKSPACES_MANAGER.write().await;
        let mut cancelation_token = CANCELATION_TOKEN.write().await;

        if workspaces_manager.is_some() || cancelation_token.is_some() {
            panic!("Workspace manager already initialized")
        }

        *workspaces_manager = Some(WorkspacesManager::default());
        drop(workspaces_manager);

        let token = CancellationToken::new();
        RT.spawn(WorkspacesManager::event_handler(token));

        *cancelation_token = Some(token);

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

    async fn event_handler(token: CancellationToken) -> anyhow::Result<()> {
        let mut events_receiver = EVENTS.subscribe();

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

                    if let Err(e) = Self::handle_event(event).await {
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

    async fn handle_event(event: HyprEvent) -> anyhow::Result<()> {
        let mut active_workspace = ACTIVE_WORKSPACE.lock().await;

        match event {
            // Смена активного workspace
            HyprEvent::WorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Deactivate(active_workspace.id),
                )
                .await?;

                active_workspace.id = id;

                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Activate(active_workspace.id),
                )
                .await?;
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
                .await?;

                active_workspace.id = workspace_id;
                active_workspace.monitor = monitor_connector;

                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Activate(active_workspace.id),
                )
                .await?;
            }
            HyprEvent::CreateWorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Create(id),
                )
                .await?;
            }
            HyprEvent::DestroyWorkspaceV2 { id, name: _ } => {
                send_event(
                    &active_workspace.monitor,
                    WorkspaceEvent::Destroy(id),
                )
                .await?;
            }
            _ => (),
        };

        Ok(())
    }

    /// Run only in main thread
    async fn dispose(&mut self) {
        let maybe_wmgr = WORKSPACES_MANAGER.write().await.take();

        if let Some(wmgr) = maybe_wmgr {
            wmgr.instances.into_iter().map(|(_, instance)| {
                if let Some(parent) = instance.widget.parent()
                    && let Some(box_parent) = parent.downcast_ref::<gtk::Box>()
                {
                    glib::idle_add_local(clone!(
                        #[strong]
                        box_parent,
                        move || {
                            box_parent.remove(&instance.widget);
                            ControlFlow::Break
                        }
                    ));
                }
            });
        }

        if let Some(token) = CANCELATION_TOKEN.write().await.take() {
            token.cancel();
        }
    }
}
