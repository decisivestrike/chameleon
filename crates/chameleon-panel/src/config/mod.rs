pub mod modules;
use arc_swap::{ArcSwap, Guard};
use chameleon_shared::utils::read_config;
pub use modules::*;

use layer_shell::Layer;
use serde::Deserialize;
use std::fmt;
use std::sync::{Arc, LazyLock, OnceLock};

/// All panel modules
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Clock,
    Battery,
    Workspaces,
    KeyboardLayout,
    Pulseaudio,
    Separator,
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
                Module::Separator => "separator",
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

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Modules {
    pub left: Vec<Module>,
    pub center: Vec<Module>,
    pub right: Vec<Module>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    pub position: Position,
    pub thickness: Option<i32>,
    pub spacing: i32,
    #[serde(with = "LayerDefinition")]
    pub layer: Layer,
    pub modules: Modules,
    pub clock: ClockRules,
    pub battery: BatteryRules,
    pub workspaces: WorkspacesRules,
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
