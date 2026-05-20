use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    /// Monitor's connector
    pub monitor: String,
    #[serde(rename = "monitorID")]
    pub monitor_id: i32,
    pub windows: i32,
    #[serde(rename = "hasfullscreen")]
    pub has_fullscreen: bool,
    #[serde(rename = "lastwindow")]
    pub last_window: String,
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,
    #[serde(rename = "ispersistent")]
    pub is_persistent: bool,
    #[serde(rename = "tiledLayout")]
    pub tiled_layout: String,
}
