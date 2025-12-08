use chameleon_config::bar::Workspaces as WorkspacesConfig;
use chameleon_ipc::hyprland::{
    self, HyprEvent, events::HyprlandService, workspace::Workspace,
};
use grapes::{
    Broadcast,
    glib::clone,
    gtk::{GestureClick, Label, Orientation, Widget},
    prelude::*,
    tokio::sync::{Mutex, mpsc},
};
use std::sync::LazyLock;

static INSTANSES: LazyLock<Mutex<Vec<(String, mpsc::Sender<WorkspaceEvent>)>>> =
    LazyLock::new(|| {
        RT.spawn(event_handler());
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

async fn event_handler() {
    let mut rx = HyprlandService::subscribe();

    let active_workspace = Workspace::active().await.unwrap();
    let mut active_workspace_id = active_workspace.id;
    let mut active_monitor = active_workspace.monitor;

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

        match maybe_event {
            Err(message) => {
                log::error!("{}", message);
                continue;
            }
            Ok(_) => (),
        };

        match maybe_event.unwrap() {
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

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Create(i32),
    Destroy(i32),
    ChangeActive { from: i32, to: i32 },
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
    type Props = &'static WorkspacesConfig;

    fn new(_config: &WorkspacesConfig) -> Self {
        let (sender, mut receiver) = mpsc::channel(16);

        let root = gtk::Box::new(Orientation::Horizontal, 0);
        root.set_widget_name("workspaces");

        let workspaces = Self { root };
        workspaces.root.connect_realize(clone!(
            #[strong]
            workspaces,
            move |_| workspaces.on_realize(&sender)
        ));

        glib::spawn_future_local(clone!(
            #[strong]
            workspaces,
            async move {
                while let Some(data) = receiver.recv().await {
                    workspaces.update(data);
                }
            }
        ));

        workspaces
    }
}

impl Workspaces {
    async fn change_active_workspace(id: i32) {
        let command = format!("dispatch workspace {}", id);
        let _ = hyprland::command(command.as_bytes()).await;
    }

    fn on_realize(&self, sender: &mpsc::Sender<WorkspaceEvent>) {
        let surface = self.root.native().unwrap().surface().unwrap();
        let monitor = self.root.display().monitor_at_surface(&surface).unwrap();

        RT.block_on(async {
            for ws in hyprland::Workspace::on_monitor(&monitor).await.unwrap() {
                self.add_workspace_button(ws.id);
            }
        });

        let connector_name = monitor.connector().unwrap().to_string();

        let mut instances = INSTANSES.blocking_lock();
        instances.push((connector_name, sender.clone()));
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

    fn find_button(&self, button_id: i32, f: impl FnOnce(Widget)) {
        let button_name = format!("button-{}", button_id);

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
        let mut maybe_child = self.root.first_child();

        while let Some(child) = maybe_child {
            let child_id: i32 = child
                .widget_name()
                .split_once('-')
                .unwrap()
                .1
                .parse()
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
}
