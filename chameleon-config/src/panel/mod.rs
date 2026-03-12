pub mod modules;
pub use modules::*;

use serde::Deserialize;
use std::fmt;

/// All panel modules
#[derive(Debug, Deserialize)]
pub enum Module {
    #[serde(rename = "clock")]
    Clock,
    #[serde(rename = "battery")]
    Battery,
    #[serde(rename = "workspaces")]
    Workspaces,
    #[serde(rename = "layout")]
    Layout,
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
                Module::Layout => "layout",
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
pub enum Position {
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
pub enum Layer {
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
pub struct PanelConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub position: Position,
    pub thickness: Option<i32>,
    #[serde(default)]
    pub spacing: i32,
    #[serde(default)]
    pub layer: Layer,
    #[serde(default)]
    pub modules_left: Vec<Module>,
    #[serde(default)]
    pub modules_center: Vec<Module>,
    #[serde(default)]
    pub modules_right: Vec<Module>,
    #[serde(default, rename = "clock")]
    pub clock: ClockConfig,
    #[serde(default, rename = "battery")]
    pub battery: BatteryConfig,
    #[serde(default, rename = "workspaces")]
    pub workspaces: WorkspacesConfig,
}
