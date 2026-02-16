mod event;
mod manager;

use crate::{
    common::Metadata,
    modules::{
        ModuleFactory,
        workspaces::{event::WorkspaceEvent, manager::WorkspacesManager},
    },
};
use anyhow::Result;
use chameleon_config::panel::WorkspacesConfig;
use chameleon_ipc::hyprland::{self, Workspace};
use grapes::{
    glib::clone::Downgrade,
    gtk::{GestureClick, Label, Widget},
    prelude::{containers::GrapesBoxExt, *},
    tokio::sync::mpsc::{self},
};
use std::rc::Rc;

#[derive(Debug, Component)]
pub struct Workspaces {
    #[root]
    root: gtk::Box,
}

impl ModuleFactory for Workspaces {
    type Config = WorkspacesConfig;

    /// Creates `Workspaces` instance and register it in `WorkspacesManager`
    fn create(
        config: &WorkspacesConfig,
        meta: &Metadata,
    ) -> Result<Rc<dyn Component>> {
        let (sender, receiver) = mpsc::channel(64);

        let workspaces = Workspaces::new(config, &meta, receiver);

        if let Err(e) =
            RT.block_on(WorkspacesManager::register(&meta.monitor, sender))
        {
            log::error!("{e}");
        };

        Ok(workspaces)
    }
}

impl Workspaces {
    fn new(
        _config: &WorkspacesConfig,
        meta: &Metadata,
        receiver: mpsc::Receiver<WorkspaceEvent>,
    ) -> Rc<Self> {
        let root = gtk::Box::new(meta.orientation, 0);
        root.set_widget_name("workspaces");

        let workspaces = Rc::new(Self { root });

        RT.block_on(async {
            for ws in Workspace::on_monitor(&meta.monitor).await.unwrap() {
                workspaces.add_workspace_button(ws.id);
            }

            let active_workspace = Workspace::active().await.unwrap();
            workspaces.activate_workspace_button(active_workspace.id);
        });

        workspaces.spawn_listener_local(receiver);

        workspaces
    }

    fn spawn_listener_local(
        self: &Rc<Self>,
        mut receiver: mpsc::Receiver<WorkspaceEvent>,
    ) {
        let ws_weak = self.downgrade();

        glib::spawn_future_local(async move {
            while let Some(data) = receiver.recv().await
                && let Some(ws) = &ws_weak.upgrade()
            {
                ws.update(data);
            }
        });

        log::debug!("Local listener stopped")
    }

    fn create_button(id: i32) -> Label {
        let button = Label::new(Some(&id.to_string()));
        let event_controller = GestureClick::new();

        event_controller.connect_pressed(move |_, _, _, _| {
            // On click
            RT.spawn(async move {
                let command = format!("dispatch workspace {}", id);
                let _ = hyprland::command(command.as_bytes()).await;
            });
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
}

/// Event handlers
impl Workspaces {
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

    fn remove_workspace_button(&self, id: i32) {
        self.find_button(id, |child| self.root.remove(&child));
    }

    fn activate_workspace_button(&self, id: i32) {
        self.find_button(id, |child| child.add_css_class("active"));
    }

    fn deactivate_workspace_button(&self, id: i32) {
        self.find_button(id, |child| child.remove_css_class("active"));
    }
}

impl UpdateableComponent for Workspaces {
    type Message = WorkspaceEvent;

    fn update(&self, event: WorkspaceEvent) {
        match event {
            WorkspaceEvent::Create(id) => self.add_workspace_button(id),
            WorkspaceEvent::Destroy(id) => self.remove_workspace_button(id),
            WorkspaceEvent::Activate(id) => self.activate_workspace_button(id),
            WorkspaceEvent::Deactivate(id) => {
                self.deactivate_workspace_button(id)
            }
        }
    }
}
