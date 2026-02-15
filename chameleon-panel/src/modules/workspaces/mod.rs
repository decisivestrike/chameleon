mod event;
pub mod factory;
mod manager;

use crate::{
    common::Metadata,
    modules::workspaces::{event::WorkspaceEvent, manager::WorkspacesManager},
};
use chameleon_config::panel::WorkspacesConfig;
use chameleon_ipc::hyprland::{self, Workspace};
use grapes::{
    glib::{
        Downgrade,
        clone::{Downgrade, Upgrade},
    },
    gtk::{GestureClick, Label, Widget},
    prelude::{containers::GrapesBoxExt, *},
    tokio::sync::mpsc::{self, Receiver},
};
use std::sync::Arc;

#[derive(Clone, Debug, Component, Downgrade)]
pub struct Workspaces {
    #[root]
    root: gtk::Box,
    monitor: Arc<String>,
}

impl Workspaces {
    fn new(
        _config: &WorkspacesConfig,
        meta: &Metadata,
        receiver: mpsc::Receiver<WorkspaceEvent>,
    ) -> Self {
        let root = gtk::Box::new(meta.orientation, 0);
        root.set_widget_name("workspaces");

        let workspaces = Self {
            root,
            monitor: meta.monitor.connector().unwrap().to_string().into(),
        };

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

    fn spawn_listener_local(&self, mut receiver: Receiver<WorkspaceEvent>) {
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

impl Drop for Workspaces {
    fn drop(&mut self) {
        if Arc::strong_count(&self.monitor) == 1 {
            log::debug!("ws unregistered");

            RT.spawn(WorkspacesManager::unregister(self.monitor.clone()));
        }
    }
}
