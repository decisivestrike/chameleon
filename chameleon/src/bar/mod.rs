pub mod modules;

use crate::bar::modules::{Battery, Clock};
use chameleon_configuration as config;
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
};
use log::info;

pub struct Bar {
    window: ApplicationWindow,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
    config: &'static config::Bar,
}

impl WindowComponent for Bar {
    type Props = &'static config::Bar;

    fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        config: Self::Props,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        let cb = gtk::CenterBox::new();
        window.set_child(Some(&cb));

        let left = gtk::Box::new(Orientation::Horizontal, config.spacing);
        let center = gtk::Box::new(Orientation::Horizontal, config.spacing);
        let right = gtk::Box::new(Orientation::Horizontal, config.spacing);

        cb.set_start_widget(Some(&left));
        cb.set_center_widget(Some(&center));
        cb.set_end_widget(Some(&right));

        let bar = Self {
            window,
            left,
            center,
            right,
            config,
        };

        bar.setup(monitor);

        let all_modules = [
            (&config.modules_left, ModulePlacement::Left),
            (&config.modules_center, ModulePlacement::Center),
            (&config.modules_right, ModulePlacement::Right),
        ];

        for (modules, placement) in all_modules {
            for name in modules {
                match name {
                    Module::Clock => {
                        let clock = Clock::new(&config.clock_config);
                        bar.add_module(clock, &placement)
                    }
                    Module::Battery => {
                        let battery = Battery::new(&config.battery_config);
                        bar.add_module(battery, &placement)
                    }
                };

                info!("Added {name} to {placement} bar group.");
            }
        }

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

    fn setup(&self, monitor: &gdk::Monitor) {
        let window = &self.window;

        window.init_layer_shell();

        window.set_widget_name("bar");
        window.set_default_width(monitor.geometry().width());

        window.set_decorated(false);
        window.set_resizable(false);

        window.set_keyboard_mode(KeyboardMode::OnDemand);

        if let Some(thickness) = self.config.thickness {
            window.set_default_height(thickness);
            window.set_exclusive_zone(thickness);
        } else {
            window.auto_exclusive_zone_enable();
        }

        window.set_layer(match self.config.layer {
            BarLayer::Background => Layer::Background,
            BarLayer::Bottom => Layer::Bottom,
            BarLayer::Top => Layer::Top,
            BarLayer::Overlay => Layer::Overlay,
        });

        self.set_position(&self.config.position);

        window.set_monitor(Some(monitor));
    }

    fn set_position(&self, position: &Position) {
        let window = &self.window;

        match position {
            Position::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_anchor(Edge::Left, true);
            }
            Position::Right => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, false);
            }
            Position::Bottom => {
                window.set_anchor(Edge::Top, false);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
            }
            Position::Left => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, false);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
            }
        }
    }
}
