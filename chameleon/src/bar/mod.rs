pub mod modules;

use crate::bar::modules::{Battery, Clock, Workspaces};
use chameleon_config::{self as config, bar::Modules};
use config::bar::{Layer as BarLayer, Module, ModulePlacement, Position};
use grapes::{
    Component, WindowComponent,
    extensions::GrapesBoxExt,
    gtk::{
        self, ApplicationWindow, Orientation,
        gdk::{self, prelude::MonitorExt},
        prelude::{GtkWindowExt, WidgetExt},
    },
    layer_shell::{Edge, KeyboardMode, Layer, LayerShell},
    prelude::BoxExt,
};
use log::info;
use std::rc::Rc;

pub struct Bar {
    window: ApplicationWindow,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
}

impl WindowComponent for Bar {
    type Props = Rc<config::Bar>;

    fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        config: Self::Props,
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
            left,
            center,
            right,
        };

        bar.setup_window(monitor);
        bar.apply_config(config);

        bar
    }

    fn present(&self) {
        self.window.present();
    }
}

impl Bar {
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

    fn apply_config(&self, config: Rc<config::Bar>) {
        let window = &self.window;

        if let Some(thickness) = config.thickness {
            window.set_default_height(thickness);
            window.set_exclusive_zone(thickness);
        } else {
            window.auto_exclusive_zone_enable();
        }

        window.set_layer(match config.layer {
            BarLayer::Background => Layer::Background,
            BarLayer::Bottom => Layer::Bottom,
            BarLayer::Top => Layer::Top,
            BarLayer::Overlay => Layer::Overlay,
        });

        self.set_position(&config.position);

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

    fn setup_window(&self, monitor: &gdk::Monitor) {
        let window = &self.window;

        window.init_layer_shell();
        window.set_namespace(Some("chameleon-taskbar"));

        window.set_widget_name("bar");
        window.set_default_width(monitor.geometry().width());

        window.set_decorated(false);
        window.set_resizable(false);

        window.set_keyboard_mode(KeyboardMode::OnDemand);

        window.set_monitor(Some(monitor));
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

    pub fn destroy(&self) {
        self.window.destroy();
    }
}
