pub mod common;
pub mod modules;
pub mod services;

use std::rc::Rc;

use crate::common::Metadata;
use crate::modules::{
    Battery, Clock, KeyboardLayout, ModuleFactory, Workspaces,
};
use chameleon_config::{self as config, PanelConfig};
use chameleon_ipc::COMPOSITOR;
use config::panel::{Layer as PanelLayer, Module, ModulePlacement, Position};
use grapes::gtk::gdk::prelude::MonitorExt;
use grapes::gtk::gdk::{self};
use grapes::gtk::prelude::{GtkWindowExt, WidgetExt};
use grapes::gtk::{self, ApplicationWindow, Orientation};
use grapes::layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use grapes::prelude::OrientableExt;
use grapes::prelude::containers::GrapesBoxExt;
use grapes::{Component, WindowComponent};

#[derive(WindowComponent)]
pub struct Panel {
    #[root]
    window: ApplicationWindow,
    centerbox: gtk::CenterBox,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
    monitor: gdk::Monitor,

    modules: Vec<Rc<dyn Component>>,
}

impl Panel {
    pub fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        config: &PanelConfig,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        let centerbox = gtk::CenterBox::new();

        window.set_child(Some(&centerbox));

        let left = gtk::Box::new(Orientation::Horizontal, 0);
        let center = gtk::Box::new(Orientation::Horizontal, 0);
        let right = gtk::Box::new(Orientation::Horizontal, 0);

        let mut bar = Self {
            window,
            centerbox,
            left,
            center,
            right,
            monitor: monitor.clone(),
            modules: Vec::new(),
        };

        bar.setup_window();
        bar.configure(config);

        bar
    }

    pub fn add_module(
        &self,
        module: impl AsRef<gtk::Widget>,
        placement: &ModulePlacement,
    ) {
        match placement {
            ModulePlacement::Left => self.left.append_ref(module),
            ModulePlacement::Center => self.center.append_ref(module),
            ModulePlacement::Right => self.right.append_ref(module),
        }
    }

    pub fn recreate_boxes(&mut self, orientation: Orientation, spacing: i32) {
        self.left = gtk::Box::new(orientation, spacing);
        self.center = gtk::Box::new(orientation, spacing);
        self.right = gtk::Box::new(orientation, spacing);

        self.centerbox.set_orientation(orientation);
        self.centerbox.set_start_widget(Some(&self.left));
        self.centerbox.set_center_widget(Some(&self.center));
        self.centerbox.set_end_widget(Some(&self.right));
    }

    pub fn configure(&mut self, config: &PanelConfig) {
        self.set_position(&config.position);

        let orientation = match config.position {
            Position::Top | Position::Bottom => {
                self.recreate_boxes(Orientation::Horizontal, config.spacing);

                if let Some(thickness) = config.thickness {
                    self.window.set_default_height(thickness);
                    self.window.set_exclusive_zone(thickness);
                } else {
                    self.window.set_default_height(-1);
                    self.window.auto_exclusive_zone_enable();
                }

                self.window
                    .set_default_width(self.monitor.geometry().width());

                Orientation::Horizontal
            }
            Position::Right | Position::Left => {
                self.recreate_boxes(Orientation::Vertical, config.spacing);

                if let Some(thickness) = config.thickness {
                    self.window.set_default_width(thickness);
                    self.window.set_exclusive_zone(thickness);
                } else {
                    self.window.set_default_width(-1);
                    self.window.auto_exclusive_zone_enable();
                }

                self.window
                    .set_default_height(self.monitor.geometry().height());

                Orientation::Vertical
            }
        };

        self.window.set_layer(match config.layer {
            PanelLayer::Background => Layer::Background,
            PanelLayer::Bottom => Layer::Bottom,
            PanelLayer::Top => Layer::Top,
            PanelLayer::Overlay => Layer::Overlay,
        });

        let all_modules = [
            (&config.modules_left, ModulePlacement::Left),
            (&config.modules_center, ModulePlacement::Center),
            (&config.modules_right, ModulePlacement::Right),
        ];

        let meta = Metadata {
            monitor: self.monitor.clone(),
            orientation,
            compositor: &COMPOSITOR,
        };

        for (modules, placement) in all_modules {
            for name in modules {
                self.append_module(name, meta.clone(), &placement, &config);
            }
        }
    }

    fn append_module(
        &mut self,
        module_name: &Module,
        meta: Metadata,
        placement: &ModulePlacement,
        config: &PanelConfig,
    ) {
        let maybe_module = match module_name {
            Module::Clock => Clock::create(&config.clock, &meta),
            Module::Battery => Battery::create(&config.battery, &meta),
            Module::Workspaces => Workspaces::create(&config.workspaces, &meta),
            Module::Keymap => KeyboardLayout::create(&(), &meta),
        };

        match maybe_module {
            Ok(module) => {
                self.add_module(module.as_ref(), placement);
                self.modules.push(module);

                log::info!("Added {module_name} to {placement} bar group.");
            }
            Err(e) => log::warn!("{e}"),
        }
    }

    fn setup_window(&self) {
        let window = &self.window;

        window.init_layer_shell();
        window.set_namespace(Some("chameleon-taskbar"));

        window.set_widget_name("panel");

        window.set_decorated(false);
        window.set_resizable(false);

        window.set_keyboard_mode(KeyboardMode::OnDemand);

        window.set_monitor(Some(&self.monitor));
    }

    fn set_position(&self, position: &Position) {
        let (top, right, bottom, left) = match position {
            Position::Top => (true, true, false, true),
            Position::Right => (true, true, true, false),
            Position::Bottom => (false, true, true, true),
            Position::Left => (true, false, true, true),
        };

        let window = &self.window;
        window.set_anchor(Edge::Top, top);
        window.set_anchor(Edge::Right, right);
        window.set_anchor(Edge::Bottom, bottom);
        window.set_anchor(Edge::Left, left);
    }
}
