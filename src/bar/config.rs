use std::fmt;

use crate::bar::modules::{battery::BatteryConfig, clock::ClockConfig};
use serde::Deserialize;

/// All taskbar modules
#[derive(Debug, Deserialize)]
pub enum BarModule {
    #[serde(rename = "clock")]
    Clock,
    #[serde(rename = "battery")]
    Battery,
}

impl fmt::Display for BarModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BarModule::Clock => "clock",
                BarModule::Battery => "battery",
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
    pub modules_left: Vec<BarModule>,
    #[serde(default)]
    pub modules_center: Vec<BarModule>,
    #[serde(default)]
    pub modules_right: Vec<BarModule>,
    #[serde(default, rename = "clock")]
    pub clock_config: ClockConfig,
    #[serde(default, rename = "battery")]
    pub battery_config: BatteryConfig,
}
