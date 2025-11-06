pub mod modules;

use crate::bar::modules::{Battery, Clock, battery::BatteryConfig, clock::ClockConfig};
use anyhow::{Result, anyhow};
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
use log::{info, warn};
use serde::Deserialize;
use std::fmt;

pub struct Bar {
    window: ApplicationWindow,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
    config: &'static BarConfig,
}

impl WindowComponent for Bar {
    type Props = &'static BarConfig;

    fn new(application: &gtk::Application, monitor: &gdk::Monitor, config: Self::Props) -> Self {
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

        all_modules.into_iter().for_each(|(modules, placement)| {
            for maybe_module in modules.iter() {
                match Bar::parse_module(&maybe_module, config) {
                    Ok(module) => {
                        bar.add_module(&module, &placement);
                        info!("Added {} to {placement} bar group.", module.name());
                    }
                    Err(e) => warn!("{e}"),
                }
            }
        });

        bar
    }

    fn present(&self) {
        self.window.present();
    }
}

impl Bar {
    pub fn add_module(&self, module: impl AsRef<gtk::Widget>, placement: &ModulePlacement) {
        match placement {
            ModulePlacement::Left => self.left.append_ref(module),
            ModulePlacement::Center => self.center.append_ref(module),
            ModulePlacement::Right => self.right.append_ref(module),
        }
    }

    fn parse_module(module_name: &String, bar_config: &'static BarConfig) -> Result<BarModule> {
        match module_name.as_str() {
            "clock" => {
                let clock = Clock::new(&bar_config.clock_config);
                Ok(BarModule::Clock(clock))
            }
            "battery" => {
                let battery = Battery::new(&bar_config.battery_config);
                Ok(BarModule::Battery(battery))
            }
            undefined_module_name => {
                let e = anyhow!("Undefined module: {undefined_module_name}");
                Err(e)
            }
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

    fn set_position(&self, position: &BarPosition) {
        let window = &self.window;

        match position {
            BarPosition::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_anchor(Edge::Left, true);
            }
            BarPosition::Right => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, false);
            }
            BarPosition::Bottom => {
                window.set_anchor(Edge::Top, false);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
            }
            BarPosition::Left => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, false);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
            }
        }
    }
}

pub enum BarModule {
    Clock(Clock),
    Battery(Battery),
}

impl BarModule {
    pub fn name(&self) -> &str {
        match self {
            BarModule::Clock(clock) => clock.name(),
            BarModule::Battery(battery) => battery.name(),
        }
    }
}

impl AsRef<gtk::Widget> for BarModule {
    fn as_ref(&self) -> &gtk::Widget {
        match self {
            BarModule::Clock(clock) => clock.as_ref(),
            BarModule::Battery(battery) => battery.as_ref(),
        }
    }
}

#[derive(Debug)]
pub enum ModulePlacement {
    Left,
    Center,
    Right,
}

impl fmt::Display for ModulePlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ModulePlacement::Left => "left",
                ModulePlacement::Center => "center",
                ModulePlacement::Right => "right",
            }
        )
    }
}

#[derive(Debug, Default, Deserialize)]
pub enum BarPosition {
    #[serde(rename = "top")]
    #[default]
    Top,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "left")]
    Left,
}

#[derive(Debug, Default, Deserialize)]
pub enum BarLayer {
    #[serde(rename = "background")]
    Background,
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "top")]
    #[default]
    Top,
    #[serde(rename = "overlay")]
    Overlay,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BarConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub position: BarPosition,
    pub thickness: Option<i32>,
    #[serde(default)]
    pub spacing: i32,
    #[serde(default)]
    pub layer: BarLayer,
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,
    #[serde(default, rename = "clock")]
    pub clock_config: ClockConfig,
    #[serde(default, rename = "battery")]
    pub battery_config: BatteryConfig,
}
