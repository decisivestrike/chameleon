pub mod modules;

use crate::modules::{Battery, Clock, Workspaces};
use chameleon_config::{self as config, PanelConfig, panel::Modules};
use config::panel::{Layer as PanelLayer, Module, ModulePlacement, Position};
use grapes::{
    WindowComponent,
    gtk::{
        self, ApplicationWindow, Orientation,
        gdk::{self, prelude::MonitorExt},
        prelude::{GtkWindowExt, WidgetExt},
    },
    layer_shell::{Edge, KeyboardMode, Layer, LayerShell},
    prelude::{OrientableExt, containers::GrapesBoxExt},
};
use log::info;
use std::rc::Rc;

#[derive(WindowComponent)]
pub struct Panel {
    #[root]
    window: ApplicationWindow,
    centerbox: gtk::CenterBox,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
    monitor: gdk::Monitor,
}

impl Panel {
    pub fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        config: Rc<PanelConfig>,
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

    pub fn configure(&mut self, config: Rc<config::PanelConfig>) {
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

        let Modules {
            clock,
            battery,
            workspaces,
        } = config.modules();

        for (modules, placement) in all_modules {
            for name in modules {
                match name {
                    Module::Clock => {
                        let clock = Clock::new(clock.clone());
                        self.add_module(clock, &placement);
                    }
                    Module::Battery => {
                        let battery = Battery::new(battery.clone());
                        self.add_module(battery, &placement);
                    }
                    Module::Workspaces => {
                        let workspaces =
                            Workspaces::new(workspaces.clone(), orientation);
                        self.add_module(workspaces, &placement);
                    }
                };

                info!("Added {name} to {placement} bar group.");
            }
        }
    }

    fn setup_window(&self) {
        let window = &self.window;

        window.init_layer_shell();
        window.set_namespace(Some("chameleon-taskbar"));

        window.set_widget_name("bar");

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
