use chameleon_config::bar::Workspaces as WorkspacesConfig;
use chameleon_ipc::hyprland::{
    self, HyprEvent, events::HyprlandService, workspace::Workspace,
};
use grapes::{
    Broadcast,
    glib::{
        Downgrade,
        clone::{self, Downgrade, Upgrade},
    },
    gtk::{GestureClick, Label, Orientation, Widget},
    prelude::*,
    tokio::sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
};
use std::{rc::Rc, sync::LazyLock};

static INSTANSES: LazyLock<Mutex<Vec<(String, mpsc::Sender<WorkspaceEvent>)>>> =
    LazyLock::new(|| {
        RT.spawn(event_handler());
        Default::default()
    });

async fn send_for_monitor(monitor_name: &String, event: WorkspaceEvent) {
    let instances = INSTANSES.lock().await;

    if let Some(pair) =
        &(*instances).iter().find(|pair| pair.0 == *monitor_name)
    {
        let sender = &pair.1;

        if let Err(e) = sender.send(event).await {
            log::error!("{e}");
        };
    } else {
        log::warn!("Cant find channel for '{}'", monitor_name);
    }
}

async fn event_handler() {
    let mut rx = HyprlandService::subscribe();

    let active_workspace = Workspace::active().await.unwrap();
    let mut active_workspace_id = active_workspace.id;
    let mut active_monitor = active_workspace.monitor;

    // TODO: on every reload
    send_for_monitor(
        &active_monitor,
        WorkspaceEvent::ChangeActive {
            from: 0,
            to: active_workspace_id,
        },
    )
    .await;

    loop {
        let maybe_event = rx.recv().await;

        let event = match maybe_event {
            Ok(event) => event,
            Err(e) => {
                log::error!("{e}");
                continue;
            }
        };

        match event {
            HyprEvent::WorkspaceV2 { id, name: _ } => {
                send_for_monitor(
                    &active_monitor,
                    WorkspaceEvent::ChangeActive {
                        from: active_workspace_id,
                        to: id,
                    },
                )
                .await;

                active_workspace_id = id;
            }
            HyprEvent::FocusedMonV2 {
                monitor_name,
                workspace_id,
            } => {
                // Тут должны быть разные мониторы
                send_for_monitor(
                    &monitor_name,
                    WorkspaceEvent::ChangeActive {
                        from: active_workspace_id,
                        to: workspace_id,
                    },
                )
                .await;

                active_workspace_id = workspace_id;
                active_monitor = monitor_name;
            }
            HyprEvent::CreateWorkspaceV2 { id, name: _ } => {
                send_for_monitor(&active_monitor, WorkspaceEvent::Create(id))
                    .await;
            }
            HyprEvent::DestroyWorkspaceV2 { id, name: _ } => {
                send_for_monitor(&active_monitor, WorkspaceEvent::Destroy(id))
                    .await;
            }
            _ => (),
        };
    }
}

struct WorkspacesManager {
    active_workspace: Workspace,
}

impl WorkspacesManager {
    /// Send event for
    async fn send_event(&self, monitor: &gdk::Monitor, event: WorkspaceEvent) {
        let instances = INSTANSES.lock().await;
        let monitor_name = monitor.connector().unwrap();

        if let Some(pair) =
            &(*instances).iter().find(|pair| pair.0 == *monitor_name)
        {
            let sender = &pair.1;

            if let Err(e) = sender.send(event).await {
                log::error!("{e}");
            };
        } else {
            log::warn!("Cant find channel for '{}'", monitor_name);
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Create(i32),
    Destroy(i32),
    ChangeActive { from: i32, to: i32 },
}

#[derive(Clone, Debug, GtkCompatible, Downgrade)]
pub struct Workspaces {
    #[root]
    root: gtk::Box,
    // active_workspace
}

impl Workspaces {
    pub fn new(
        _config: Rc<WorkspacesConfig>,
        orientation: Orientation,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(16);

        let root = gtk::Box::new(orientation, 0);
        root.set_widget_name("workspaces");

        let workspaces = Self { root: root.clone() };

        workspaces.connect_handlers(sender);
        workspaces.spawn_listener_local(receiver);

        workspaces
    }

    fn connect_handlers(&self, sender: Sender<WorkspaceEvent>) {
        let root = &self.root;
        root.connect_unrealize(Self::on_unrealize);

        let ws_weak = self.downgrade();
        root.connect_realize(move |_| Self::on_realize(&ws_weak, &sender));
    }

    fn spawn_listener_local(&self, mut receiver: Receiver<WorkspaceEvent>) {
        let ws_weak = self.downgrade();

        glib::spawn_future_local(async move {
            while let Some(data) = receiver.recv().await
                && let Some(ws) = &ws_weak.upgrade()
            {
                ws.update(data);
            }
        });
    }

    fn on_realize(ws: &WorkspacesWeak, sender: &mpsc::Sender<WorkspaceEvent>) {
        let workspaces = clone::Upgrade::upgrade(ws).unwrap();

        let surface = workspaces.root.native().unwrap().surface().unwrap();
        let monitor = workspaces
            .root
            .display()
            .monitor_at_surface(&surface)
            .unwrap();
        let connector_name = monitor.connector().unwrap().to_string();

        RT.block_on(async {
            for ws in hyprland::Workspace::on_monitor(&monitor).await.unwrap() {
                workspaces.add_workspace_button(ws.id);
            }
        });

        let mut instances = INSTANSES.blocking_lock();
        instances.push((connector_name, sender.clone()));
    }

    fn on_unrealize(root: &gtk::Box) {
        let surface = root.native().expect("can get native").surface().unwrap();
        let monitor = root.display().monitor_at_surface(&surface).unwrap();
        let connector_name = monitor.connector().unwrap().to_string();

        let mut instances = INSTANSES.blocking_lock();
        instances.retain(|(c, _)| *c != connector_name);
    }

    async fn change_active_workspace(id: i32) {
        let command = format!("dispatch workspace {}", id);
        let _ = hyprland::command(command.as_bytes()).await;
    }

    fn create_button(id: i32) -> Label {
        let button = Label::new(Some(&id.to_string()));
        let event_controller = GestureClick::new();

        event_controller.connect_pressed(move |_, _, _, _| {
            RT.spawn(Self::change_active_workspace(id));
        });

        button.add_controller(event_controller);
        button.set_widget_name(&id.to_string());

        button
    }

    fn find_button(&self, button_id: i32, f: impl FnOnce(Widget)) {
        let button_name = button_id.to_string();

        self.root
            .children()
            .find(|child| child.widget_name() == button_name)
            .map(|child| f(child));
    }

    fn change_active_workspace_button(&self, from: i32, to: i32) {
        self.find_button(to, |child| child.add_css_class("active"));
        self.find_button(from, |child| child.remove_css_class("active"));
    }

    fn remove_workspace(&self, id: i32) {
        self.find_button(id, |child| self.root.remove(&child));
    }

    pub fn add_workspace_button(&self, id: i32) {
        let button = Self::create_button(id);

        for child in self.root.children() {
            let child_id: i32 = child.widget_name().parse().unwrap();

            if child_id > id {
                let prev = child.prev_sibling();
                self.root.insert_child_after(&button, prev.as_ref());
                return;
            }
        }

        self.root.append(&button)
    }
}

impl Updateable for Workspaces {
    type Message = WorkspaceEvent;

    fn update(&self, event: WorkspaceEvent) {
        match event {
            WorkspaceEvent::Create(id) => self.add_workspace_button(id),
            WorkspaceEvent::Destroy(id) => self.remove_workspace(id),
            WorkspaceEvent::ChangeActive { from, to } => {
                self.change_active_workspace_button(from, to)
            }
        }
    }
}

impl Component for Workspaces {
    const NAME: &str = "workspaces";
}
