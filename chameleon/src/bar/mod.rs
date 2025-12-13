pub mod modules;

use crate::bar::modules::{Battery, Clock, Workspaces};
use chameleon_config::{self as config, bar::Modules};
use config::bar::{Layer as BarLayer, Module, ModulePlacement, Position};
use grapes::{
    WindowComponent,
    extensions::GrapesBoxExt,
    gtk::{
        self, ApplicationWindow, Orientation,
        gdk::{self, prelude::MonitorExt},
        prelude::{GtkWindowExt, WidgetExt},
    },
    layer_shell::{Edge, KeyboardMode, Layer, LayerShell},
    prelude::{BoxExt, OrientableExt},
};
use log::info;
use std::rc::Rc;

pub trait AsBarModule {
    fn as_module(&self) -> Module;
}

// #[derive(WindowComponent)]
pub struct Bar {
    // #[window]
    window: ApplicationWindow,
    centerbox: gtk::CenterBox,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
    monitor: gdk::Monitor,
}

impl WindowComponent for Bar {
    fn present(&self) {
        self.window.present();
    }

    fn destroy(&self) {
        self.window.destroy();
    }
}

impl Bar {
    pub fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        config: Rc<config::Bar>,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        let centerbox = gtk::CenterBox::new();

        window.set_child(Some(&centerbox));

        let left = gtk::Box::new(Orientation::Horizontal, 0);
        let center = gtk::Box::new(Orientation::Horizontal, 0);
        let right = gtk::Box::new(Orientation::Horizontal, 0);

        centerbox.set_start_widget(Some(&left));
        centerbox.set_center_widget(Some(&center));
        centerbox.set_end_widget(Some(&right));

        let bar = Self {
            window,
            centerbox,
            left,
            center,
            right,
            monitor: monitor.clone(),
        };

        bar.setup_window();
        bar.apply_config(config);

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

    pub fn set_orientation(&self, orientation: Orientation) {
        self.centerbox.set_orientation(orientation);
        self.left.set_orientation(orientation);
        self.center.set_orientation(orientation);
        self.right.set_orientation(orientation);
    }

    pub fn apply_config(&self, config: Rc<config::Bar>) {
        let window = &self.window;
        window.set_exclusive_zone(0);
        self.set_position(&config.position);

        match config.position {
            Position::Top | Position::Bottom => {
                self.set_orientation(Orientation::Horizontal);

                if let Some(thickness) = config.thickness {
                    window.set_default_height(thickness);
                    window.set_exclusive_zone(thickness);
                } else {
                    window.set_default_height(-1);
                    window.auto_exclusive_zone_enable();
                }

                window.set_default_width(self.monitor.geometry().width());
            }
            Position::Right | Position::Left => {
                self.set_orientation(Orientation::Vertical);

                if let Some(thickness) = config.thickness {
                    window.set_default_width(thickness);
                    window.set_exclusive_zone(thickness);
                } else {
                    window.set_default_width(-1);
                    window.auto_exclusive_zone_enable();
                }

                window.set_default_height(self.monitor.geometry().height());
            }
        }

        window.set_layer(match config.layer {
            BarLayer::Background => Layer::Background,
            BarLayer::Bottom => Layer::Bottom,
            BarLayer::Top => Layer::Top,
            BarLayer::Overlay => Layer::Overlay,
        });

        self.left.set_spacing(config.spacing);
        self.center.set_spacing(config.spacing);
        self.right.set_spacing(config.spacing);

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

        // FIXME: duplication of modules is possible
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
                        let workspaces = Workspaces::new(workspaces.clone());
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
