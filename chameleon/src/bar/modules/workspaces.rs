use chameleon_config::bar::Workspaces as WorkspacesConfig;
use chameleon_ipc::hyprland::{
    self, HyprEvent, events::HyprlandService, workspace::Workspace,
};
use grapes::{
    Component, GtkCompatible, RT, Service, Updateable,
    glib::{self, object::Cast},
    gtk::{
        self, GestureClick, Label, Orientation,
        gdk::prelude::{DisplayExt, MonitorExt},
        prelude::{BoxExt, NativeExt, WidgetExt},
    },
    tokio::sync::{Mutex, mpsc},
};
use std::sync::LazyLock;

static INSTANSES: LazyLock<Mutex<Vec<(String, mpsc::Sender<WorkspaceEvent>)>>> =
    LazyLock::new(|| {
        RT.spawn(async move { workspace_bgh().await });
        Default::default()
    });

async fn send_for_monitor(monitor_name: &String, event: WorkspaceEvent) {
    let instances = INSTANSES.lock().await;

    let sender = &(*instances)
        .iter()
        .find(|i| i.0 == *monitor_name)
        .unwrap()
        .1;

    sender.send(event).await.unwrap();
}

// Don't subscribe on it
async fn workspace_bgh() {
    let mut rx = HyprlandService::subscribe();

    let active_workspace = Workspace::active().await.unwrap();
    let mut active_workspace_id = active_workspace.id;
    let mut active_monitor = active_workspace.monitor;

    send_for_monitor(
        &active_monitor,
        WorkspaceEvent::Active(active_workspace_id),
    )
    .await;

    loop {
        let maybe_event = rx.recv().await;

        match maybe_event {
            Err(message) => {
                log::error!("{}", message);
                continue;
            }
            Ok(_) => (),
        };

        match maybe_event.unwrap() {
            HyprEvent::WorkspaceV2 { id, name: _ } => {
                send_for_monitor(&active_monitor, WorkspaceEvent::Active(id))
                    .await;
                send_for_monitor(
                    &active_monitor,
                    WorkspaceEvent::Unactive(active_workspace_id),
                )
                .await;

                active_workspace_id = id;
            }
            HyprEvent::FocusedMonV2 {
                monitor_name,
                workspace_id,
            } => {
                send_for_monitor(
                    &monitor_name,
                    WorkspaceEvent::Active(workspace_id),
                )
                .await;
                send_for_monitor(
                    &monitor_name,
                    WorkspaceEvent::Unactive(active_workspace_id),
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

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Create(i32),
    Destroy(i32),
    Active(i32),
    Unactive(i32),
}

#[derive(Clone, Debug, GtkCompatible)]
pub struct Workspaces {
    #[root]
    root: gtk::Box,
}

impl Updateable for Workspaces {
    type Message = WorkspaceEvent;

    fn update(&self, event: WorkspaceEvent) {
        match event {
            WorkspaceEvent::Create(id) => self.add_workspace(id),
            WorkspaceEvent::Destroy(id) => self.remove_workspace(id),
            WorkspaceEvent::Active(id) => self.set_active(id),
            WorkspaceEvent::Unactive(id) => self.set_unactive(id),
        }
    }
}

impl Component for Workspaces {
    const NAME: &str = "workspaces";
    type Props = &'static WorkspacesConfig;

    fn new(_config: &WorkspacesConfig) -> Self {
        let (sender, mut receiver) = mpsc::channel(16);

        let root = gtk::Box::new(Orientation::Horizontal, 0);
        let root_clone = root.clone();

        let workspaces = Self { root };

        {
            let workspaces_clone = workspaces.clone();

            root_clone.connect_realize(move |root| {
                let surface = root.native().unwrap().surface().unwrap();
                let monitor =
                    root.display().monitor_at_surface(&surface).unwrap();

                RT.block_on(async {
                    for ws in
                        hyprland::Workspace::on_monitor(&monitor).await.unwrap()
                    {
                        workspaces_clone.add_workspace(ws.id);
                    }
                });

                let connector_name = monitor.connector().unwrap().to_string();

                {
                    let mut instances = INSTANSES.blocking_lock();
                    let sender_clone = sender.clone();
                    instances.push((connector_name, sender_clone));
                }
            });
        }

        {
            let workspaces_clone = workspaces.clone();

            glib::spawn_future_local(async move {
                while let Some(data) = receiver.recv().await {
                    workspaces_clone.update(data);
                }
            });
        }

        workspaces
    }
}

impl Workspaces {
    async fn change_active_workspace(id: i32) {
        let command = format!("dispatch workspace {}", id);
        hyprland::command(command.as_bytes()).await.unwrap();
    }

    fn create_button(id: i32) -> Label {
        let button = Label::new(Some(&id.to_string()));
        let event_controller = GestureClick::new();

        event_controller.connect_pressed(move |_, _, _, _| {
            RT.spawn(Self::change_active_workspace(id));
        });

        button.add_controller(event_controller);
        button.set_widget_name(&format!("button-{}", id));

        button
    }

    fn set_active(&self, button_id: i32) {
        let mut maybe_child = self.root.first_child();
        let button_name = format!("button-{}", button_id);

        while let Some(child) = maybe_child {
            if child.widget_name() == button_name {
                child.add_css_class("active");
            }

            maybe_child = child.next_sibling();
        }
    }

    fn set_unactive(&self, button_id: i32) {
        let mut maybe_child = self.root.first_child();
        let button_name = format!("button-{}", button_id);

        while let Some(child) = maybe_child {
            if child.widget_name() == button_name {
                child.remove_css_class("active");
            }

            maybe_child = child.next_sibling();
        }
    }

    pub fn add_workspace(&self, id: i32) {
        let button = Self::create_button(id);
        let mut maybe_child = self.root.first_child();

        while let Some(child) = maybe_child {
            let child_id = child
                .clone()
                .downcast::<Label>()
                .unwrap()
                .label()
                .parse::<i32>()
                .unwrap();

            if child_id > id {
                let prev = child.prev_sibling();
                self.root.insert_child_after(&button, prev.as_ref());
                return;
            }

            maybe_child = child.next_sibling();
        }

        self.root.append(&button)
    }

    fn remove_workspace(&self, id: i32) {
        let mut maybe_child = self.root.first_child();
        let button_name = format!("button-{}", id);

        while let Some(child) = maybe_child {
            if child.widget_name() == button_name {
                self.root.remove(&child);
                break;
            }

            maybe_child = child.next_sibling();
        }
    }
}
