pub mod modules;
pub use modules::*;

use layer_shell::Layer;
use serde::Deserialize;
use std::fmt;
use std::sync::OnceLock;

pub static CONFIG: OnceLock<Rules> = OnceLock::new();

pub fn config() -> &'static Rules {
    CONFIG
        .get()
        .expect("Configuration must be loaded at the start")
}

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

#[doc(hidden)]
#[derive(Default, Deserialize)]
#[serde(remote = "Layer", rename_all = "snake_case")]
enum LayerDefinition {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Modules {
    pub left: Vec<Module>,
    pub center: Vec<Module>,
    pub right: Vec<Module>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    pub position: Position,
    pub thickness: Option<i32>,
    pub spacing: i32,
    #[serde(with = "LayerDefinition")]
    pub layer: Layer,
    pub modules: Modules,
    pub clock: ClockConfig,
    pub battery: BatteryConfig,
    pub workspaces: WorkspacesConfig,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            position: Default::default(),
            thickness: Default::default(),
            spacing: Default::default(),
            layer: Layer::Top,
            modules: Default::default(),
            clock: Default::default(),
            battery: Default::default(),
            workspaces: Default::default(),
        }
    }
}
