mod event;
mod factory;
mod manager;

use crate::{common::Metadata, modules::workspaces::event::WorkspaceEvent};
use chameleon_config::panel::WorkspacesConfig;
use chameleon_ipc::hyprland::{self};
use grapes::{
    glib::{
        Downgrade,
        clone::{Downgrade, Upgrade},
    },
    gtk::{GestureClick, Label, Widget},
    prelude::{containers::GrapesBoxExt, *},
    tokio::sync::mpsc::{self, Receiver, Sender},
};
use std::rc::Rc;

#[derive(Clone, Debug, Component, Downgrade)]
pub struct Workspaces {
    #[root]
    root: gtk::Box,
}

impl Workspaces {
    pub fn new(
        config: Rc<WorkspacesConfig>,
        meta: Metadata,
        receiver: mpsc::Receiver<WorkspaceEvent>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(16);

        let root = gtk::Box::new(meta.orientation, 0);
        root.set_widget_name("workspaces");

        let workspaces = Self { root };

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
        let workspaces = ws.upgrade().unwrap();

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

        INSTANSES.insert(connector_name, sender.clone());
    }

    fn on_unrealize(root: &gtk::Box) {
        let surface = root.native().expect("can get native").surface().unwrap();
        let monitor = root.display().monitor_at_surface(&surface).unwrap();
        let connector_name = monitor.connector().unwrap().to_string();

        INSTANSES.retain(|c, _| *c != connector_name);
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

impl UpdateableComponent for Workspaces {
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
