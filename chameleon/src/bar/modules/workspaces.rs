use chameleon_config::bar::Workspaces as WorkspacesConfig;
use chameleon_ipc::hyprland::{HyprEvent, events::HyprlandService};
use grapes::{
    Component, Connectable, GtkCompatible, Reactive, Updateable,
    extensions::GrapesBoxExt,
    gtk::{self, Label, Orientation},
    state,
    tokio::sync::mpsc,
};
use std::sync::{LazyLock, Mutex};

static INSTANSES: LazyLock<Mutex<Vec<(String, mpsc::Sender<HyprEvent>)>>> =
    LazyLock::new(|| Default::default());

#[derive(Clone, Debug, GtkCompatible)]
pub struct Workspaces {
    #[root]
    container: gtk::Box,
}

impl Updateable for Workspaces {
    type Message = HyprEvent;

    fn update(&self, message: HyprEvent) {
        match message {
            HyprEvent::ActiveLayout {
                keyboard_name,
                layout_name,
            } => todo!(),
            HyprEvent::WorkspaceV2 { id, name } => todo!(),
            HyprEvent::FocusedMonV2 {
                monitor_name,
                workspace_id,
            } => todo!(),
            HyprEvent::MonitorRemoved { name } => todo!(),
            HyprEvent::MonitorAdded { name } => todo!(),
            HyprEvent::CreateWorkspaceV2 { id, name } => todo!(),
            HyprEvent::DestroyWorkspaceV2 { id, name } => todo!(),
        }
    }
}

impl Component for Workspaces {
    const NAME: &str = "workspaces";
    type Props = &'static WorkspacesConfig;

    fn new(config: &WorkspacesConfig) -> Self {
        // let (sender, mut receiver) = mpsc::channel(16);
        let container = gtk::Box::new(Orientation::Horizontal, 0);

        let current_event = state("".to_string());
        current_event
            .connect_service_unmatched::<HyprlandService>(|e| e.to_string());

        let label = Label::statefull(&current_event);
        container.append_ref(label);

        Self { container }
    }
}
