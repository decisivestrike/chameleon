pub mod modules;
pub use modules::*;

use serde::Deserialize;
use std::fmt;

/// All panel modules
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Clock,
    Battery,
    Workspaces,
    KeyboardLayout,
    Pulseaudio,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Module::Clock => "clock",
                Module::Battery => "battery",
                Module::Workspaces => "workspaces",
                Module::KeyboardLayout => "keyboard layout",
                Module::Pulseaudio => "pulseaudio",
            }
        )
    }
}

/// Placement on taskbar
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
#[serde(rename_all = "snake_case")]
pub enum Position {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PanelConfig {
    pub enabled: bool,
    pub position: Position,
    pub thickness: Option<i32>,
    pub spacing: i32,
    pub layer: Layer,
    pub modules_left: Vec<Module>,
    pub modules_center: Vec<Module>,
    pub modules_right: Vec<Module>,
    pub clock: ClockConfig,
    pub battery: BatteryConfig,
    pub workspaces: WorkspacesConfig,
}
