mod event;
mod manager;

use crate::config::WorkspacesRules;
use crate::modules::workspaces::hyprland::event::WorkspaceEvent;
use crate::modules::workspaces::hyprland::manager::{
    MANAGER, WorkspacesManager,
};
use crate::modules::{Metadata, PanelModule};
use anyhow::{Result, bail};
use chameleon_hyprland::Hyprland;
use chameleon_hyprland::entities::Workspace;
use chameleon_ipc::{COMPOSITOR, Compositor};
use glib::clone::Downgrade;
use gtk::gdk::prelude::MonitorExt;
use gtk::glib::Object;
use gtk::glib::object::Cast;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use gtk::{GestureClick, Label, Widget, glib};
use gtkio::RUNTIME;
use std::iter::successors;
use std::rc::Rc;
use tokio::sync::mpsc::{self};
use tracing::{debug, error};

const SPECIAL_WORKSPACE_ID: i32 = -98;

mod imp {
    use std::sync::OnceLock;

    use chameleon_hyprland::Hyprland;
    use gtk::glib;
    use gtk::prelude::WidgetExt;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct HyprlandWorkspacesImp {
        pub hyprland: OnceLock<&'static Hyprland>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HyprlandWorkspacesImp {
        const NAME: &'static str = "HyprlandWorkspaces";
        type Type = super::HyprlandWorkspaces;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for HyprlandWorkspacesImp {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_widget_name("workspaces");
        }
    }

    impl WidgetImpl for HyprlandWorkspacesImp {}

    impl BoxImpl for HyprlandWorkspacesImp {}
}

glib::wrapper! {
    /// Panel
    pub struct HyprlandWorkspaces(ObjectSubclass<imp::HyprlandWorkspacesImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PanelModule for HyprlandWorkspaces {
    type Rules = WorkspacesRules;

    fn create(rules: Self::Rules, meta: Rc<Metadata>) -> Result<Widget> {
        if let Compositor::Hyprland(hyprland) = &*COMPOSITOR {
            let manager =
                MANAGER.get_or_init(|| WorkspacesManager::new(hyprland));

            let (sender, receiver) = mpsc::channel(64);
            let workspaces =
                HyprlandWorkspaces::new(&rules, &meta, receiver, hyprland);

            if let Err(e) =
                RUNTIME.block_on(manager.register(&meta.monitor, sender))
            {
                error!("{e}");
            };

            Ok(workspaces.upcast())
        } else {
            bail!("U can use this module only with Hyprland");
        }
    }
}

impl HyprlandWorkspaces {
    pub fn new(
        _config: &WorkspacesRules,
        meta: &Metadata,
        receiver: mpsc::Receiver<WorkspaceEvent>,
        hyprland: &'static Hyprland,
    ) -> Self {
        let workspaces: Self = Object::builder().build();
        workspaces.set_orientation(meta.orientation);
        workspaces.imp().hyprland.set(hyprland).unwrap();

        RUNTIME.block_on(async {
            let wss: Vec<Workspace> = hyprland
                .workspaces()
                .await
                .unwrap()
                .into_iter()
                .filter(|w| {
                    w.monitor == meta.monitor.connector().unwrap().to_string()
                })
                .collect();

            for ws in wss {
                if ws.id != SPECIAL_WORKSPACE_ID {
                    workspaces.add_workspace_button(ws.id);
                }
            }

            let active_workspace = hyprland.active_workspace().await.unwrap();
            workspaces.activate_workspace_button(active_workspace.id);
        });

        workspaces.spawn_listener_local(receiver);

        workspaces
    }

    fn spawn_listener_local(
        &self,
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

        debug!("Local listener stopped")
    }

    fn create_button(&self, id: i32) -> Label {
        // let WorkspacesRules {
        //     numeral_system,
        //     numeral_variant,
        // } = self.config;

        // let label = id
        //     .to_numeral(numeral_system.into(), vec![numeral_variant.into()])
        //     .unwrap();

        let label = id.to_string();

        let button = Label::new(Some(&label));
        let event_controller = GestureClick::new();

        let hyprland = *self.imp().hyprland.get().unwrap();

        event_controller.connect_pressed(move |_, _, _, _| {
            // On click
            RUNTIME.spawn(async move {
                let command =
                    format!(r#"hl.dsp.focus({{ workspace = "{id}" }})"#);

                hyprland.dispatch(&command).await.expect("should work");
            });
        });

        button.add_controller(event_controller);
        button.set_widget_name(&id.to_string());

        button
    }

    fn find_button(&self, button_id: i32, f: impl FnOnce(Widget)) {
        let button_name = button_id.to_string();

        self.children()
            .find(|child| child.widget_name() == button_name)
            .map(|child| f(child));
    }

    fn children(&self) -> impl Iterator<Item = Widget> {
        successors(self.first_child(), |child| child.next_sibling())
    }

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

/// Event handlers
impl HyprlandWorkspaces {
    pub fn add_workspace_button(&self, id: i32) {
        let button = self.create_button(id);

        for child in self.children() {
            let child_id: i32 = child.widget_name().parse().unwrap();

            if child_id > id {
                let prev = child.prev_sibling();
                self.insert_child_after(&button, prev.as_ref());
                return;
            }
        }

        self.append(&button)
    }

    fn remove_workspace_button(&self, id: i32) {
        self.find_button(id, |child| self.remove(&child));
    }

    fn activate_workspace_button(&self, id: i32) {
        self.find_button(id, |child| child.add_css_class("active"));
    }

    fn deactivate_workspace_button(&self, id: i32) {
        self.find_button(id, |child| child.remove_css_class("active"));
    }
}
