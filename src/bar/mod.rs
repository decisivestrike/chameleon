pub mod modules;

use crate::bar::modules::clock::ClockConfig;
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
use serde::Deserialize;

pub struct Bar {
    window: ApplicationWindow,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
}

impl WindowComponent for Bar {
    type Props = &'static BarConfig;

    fn new(application: &gtk::Application, monitor: &gdk::Monitor, props: Self::Props) -> Self {
        let BarConfig {
            spacing,
            thickness,
            modules_left,
            modules_center,
            modules_right,
            clock_config,
            ..
        } = props;

        let window = ApplicationWindow::new(application);
        let cb = gtk::CenterBox::new();
        window.set_child(Some(&cb));

        let left = gtk::Box::new(Orientation::Horizontal, *spacing);
        let center = gtk::Box::new(Orientation::Horizontal, *spacing);
        let right = gtk::Box::new(Orientation::Horizontal, *spacing);

        cb.set_start_widget(Some(&left));
        cb.set_center_widget(Some(&center));
        cb.set_end_widget(Some(&right));

        let bar = Self {
            window,
            left,
            center,
            right,
        };

        bar.setup(monitor, *thickness);

        for maybe_module in modules_left.iter() {
            let module = match maybe_module.as_str() {
                "clock" => modules::Clock::new(clock_config),
                _ => panic!("Undefined module: {maybe_module}"),
            };

            bar.add_module(module, Placement::Left);
            info!("Added module to left bar group.");
        }

        for maybe_module in modules_center.iter() {
            let module = match maybe_module.as_str() {
                "clock" => modules::Clock::new(clock_config),
                _ => panic!("Undefined module: {maybe_module}"),
            };

            bar.add_module(module, Placement::Center);
            info!("Added module to center bar group.");
        }

        for maybe_module in modules_right.iter() {
            let module = match maybe_module.as_str() {
                "clock" => modules::Clock::new(clock_config),
                _ => panic!("Undefined module: {maybe_module}"),
            };

            bar.add_module(module, Placement::Right);
            info!("Added module to right bar group.");
        }

        bar
    }

    fn present(&self) {
        self.window.present();
    }
}

impl Bar {
    pub fn add_module(&self, module: impl AsRef<gtk::Widget>, placement: Placement) {
        match placement {
            Placement::Left => self.left.append_ref(module),
            Placement::Center => self.center.append_ref(module),
            Placement::Right => self.right.append_ref(module),
        }
    }

    fn setup(&self, monitor: &gdk::Monitor, thickness: Option<i32>) {
        let window = &self.window;

        window.init_layer_shell();

        window.set_widget_name("bar");
        window.set_default_width(monitor.geometry().width());

        window.set_decorated(false);
        window.set_resizable(false);

        window.set_keyboard_mode(KeyboardMode::OnDemand);

        if let Some(thickness) = thickness {
            window.set_default_height(thickness);
            window.set_exclusive_zone(thickness);
        } else {
            window.auto_exclusive_zone_enable();
        }

        window.set_layer(Layer::Top);

        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        // window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);

        window.set_monitor(Some(monitor));
    }
}

pub enum Position {
    Top,
    Right,
    Bottom,
    Left,
}

pub enum Placement {
    Left,
    Center,
    Right,
}

#[derive(Debug, Default, Deserialize)]
pub enum BarLayer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BarConfig {
    #[serde(default)]
    pub enabled: bool,
    pub thickness: Option<i32>,
    #[serde(default)]
    pub spacing: i32,
    #[serde(default, rename = "layer")]
    pub layer: BarLayer,
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,
    #[serde(default, rename = "clock")]
    pub clock_config: ClockConfig,
}
