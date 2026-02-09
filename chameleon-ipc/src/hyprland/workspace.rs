use crate::hyprland::query;
use anyhow::Result;
use grapes::gtk::gdk::{Monitor, prelude::MonitorExt};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    /// Monitors connector
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

    pub async fn all() -> Result<Vec<Workspace>> {
        let json_str = query(b"j/workspaces\0").await?;

        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn on_monitor(monitor: &Monitor) -> Result<Vec<Workspace>> {
        Ok(Workspace::all()
            .await?
            .into_iter()
            .filter(|w| w.monitor == monitor.connector().unwrap().to_string())
            .collect())
    }
}
