use anyhow::Result;
use serde::Deserialize;

use crate::core::hyprland::query;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub monitor: String,
    #[serde(rename = "monitorID")]
    monitor_id: i32,
    windows: i32,
    #[serde(rename = "hasfullscreen")]
    has_fullscreen: bool,
    #[serde(rename = "lastwindow")]
    last_window: String,
    #[serde(rename = "lastwindowtitle")]
    last_window_title: String,
    #[serde(rename = "ispersistent")]
    is_persistent: bool,
}

impl Workspace {
    pub async fn active() -> Result<Workspace> {
        let json_str = query(b"j/activeworkspace\0").await?;

        Ok(serde_json::from_str(&json_str)?)
    }
}
