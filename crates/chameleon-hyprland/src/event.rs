use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

/// Contains all Hyprland events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyprEvent {
    /// Emitted when a user requests a workspace change, not on mouse movements
    /// (see `focusedmon`).
    Workspace { workspace_name: String },

    /// Emitted when a user requests a workspace change, not on mouse movements
    /// (see `focusedmon`). Includes workspace ID and name.
    WorkspaceV2 { id: i32, name: String },

    /// Emitted when the active monitor changes. Provides monitor name and
    /// workspace name.
    FocusedMon {
        monitor_name: String,
        workspace_name: String,
    },

    /// Emitted when the active monitor changes. Provides monitor connector name
    /// and workspace ID.
    FocusedMonV2 {
        monitor_connector: String,
        workspace_id: i32,
    },

    /// Emitted when the active window changes. Contains window class and title.
    ActiveWindow {
        window_class: String,
        window_title: String,
    },

    /// Emitted when the active window changes. Contains window address.
    ActiveWindowV2 { window_address: String },

    /// Emitted when a window's fullscreen status changes. `state` is 0 (exit
    /// fullscreen) or 1 (enter fullscreen).
    Fullscreen { state: bool },

    /// Emitted when a monitor is removed (disconnected).
    MonitorRemoved { name: String },

    /// Emitted when a monitor is removed (disconnected). Contains monitor ID,
    /// name, and description.
    MonitorRemovedV2 {
        id: i32,
        name: String,
        description: String,
    },

    /// Emitted when a monitor is added (connected).
    MonitorAdded { name: String },

    /// Emitted when a monitor is added (connected). Contains monitor ID, name,
    /// and description.
    MonitorAddedV2 {
        id: i32,
        name: String,
        description: String,
    },

    /// Emitted when a workspace is created.
    CreateWorkspace { name: String },

    /// Emitted when a workspace is created. Includes workspace ID and name.
    CreateWorkspaceV2 { id: i32, name: String },

    /// Emitted when a workspace is destroyed.
    DestroyWorkspace { name: String },

    /// Emitted when a workspace is destroyed. Includes workspace ID and name.
    DestroyWorkspaceV2 { id: i32, name: String },

    /// Emitted when a workspace is moved to a different monitor. Contains
    /// workspace name and monitor name.
    MoveWorkspace {
        workspace_name: String,
        monitor_name: String,
    },

    /// Emitted when a workspace is moved to a different monitor. Contains
    /// workspace ID, workspace name, and monitor name.
    MoveWorkspaceV2 {
        id: i32,
        workspace_name: String,
        monitor_name: String,
    },

    /// Emitted when a workspace is renamed. Provides workspace ID and the new
    /// name.
    RenameWorkspace { id: i32, new_name: String },

    /// Emitted when the special workspace on a monitor changes. Closing results
    /// in an empty workspace name. Contains workspace name and monitor name.
    ActiveSpecial {
        workspace_name: String,
        monitor_name: String,
    },

    /// Emitted when the special workspace on a monitor changes. Closing results
    /// in empty workspace ID and name. Contains workspace ID, workspace name,
    /// and monitor name.
    ActiveSpecialV2 {
        id: i32,
        workspace_name: String,
        monitor_name: String,
    },

    /// Emitted on a layout change of the active keyboard. Contains keyboard
    /// name and layout name.
    ActiveLayout {
        keyboard_name: String,
        layout_name: String,
    },

    /// Emitted when a window is opened. Contains window address, workspace
    /// name, window class, and window title.
    OpenWindow {
        window_address: String,
        workspace_name: String,
        window_class: String,
        window_title: String,
    },
    /// Emitted when a window is closed.
    CloseWindow { window_address: String },

    /// Emitted when a window is killed (via hyprctl kill).
    Kill { window_address: String },

    /// Emitted when a window is moved to a workspace. Contains window address
    /// and workspace name.
    MoveWindow {
        window_address: String,
        workspace_name: String,
    },

    /// Emitted when a window is moved to a workspace. Contains window address,
    /// workspace ID, and workspace name.
    MoveWindowV2 {
        window_address: String,
        workspace_id: i32,
        workspace_name: String,
    },

    /// Emitted when a layerSurface is mapped.
    OpenLayer { namespace: String },

    /// Emitted when a layerSurface is unmapped.
    CloseLayer { namespace: String },

    /// Emitted when a keybind submap changes. Empty submap name means default.
    Submap { submap_name: String },

    /// Emitted when a window changes its floating mode. `floating` is 0 (not
    /// floating) or 1 (floating).
    ChangeFloatingMode {
        window_address: String,
        floating: bool,
    },

    /// Emitted when a window requests an urgent state.
    Urgent { window_address: String },

    /// Emitted when a screencopy state of a client changes. Multiple clients
    /// may exist. `state` is 0 or 1, `owner` indicates monitor, window, or
    /// region.
    Screencast { state: bool, owner: String },

    /// Emitted when a screencopy state of a client changes. Multiple clients
    /// may exist. `state` is 0 or 1, `owner` indicates monitor, window, or
    /// region, and `name` identifies the shared target (monitor name or window
    /// title).
    ScreencastV2 {
        state: bool,
        owner: String,
        name: String,
    },
    /// Emitted when a window title changes. Contains window address.
    WindowTitle { window_address: String },

    /// Emitted when a window title changes. Contains window address and the new
    /// window title.
    WindowTitleV2 {
        window_address: String,
        window_title: String,
    },

    /// Emitted when the `togglegroup` command is used. `state` is 0 (group
    /// destroyed) or 1 (group created). `handles` contains one or more
    /// comma-separated window addresses of the group members.
    ToggleGroup { state: bool, handles: String },

    /// Emitted when a window is merged into a group. Contains the address of
    /// the merged window.
    MoveIntoGroup { window_address: String },

    /// Emitted when a window is removed from a group. Contains the address of
    /// the removed window.
    MoveOutOfGroup { window_address: String },

    /// Emitted when `ignoregrouplock` is toggled. `state` is 0 or 1.
    IgnoreGroupLock { state: bool },

    /// Emitted when `lockgroups` is toggled. `state` is 0 or 1.
    LockGroups { state: bool },

    /// Emitted when the config is done reloading. Has no data.
    ConfigReloaded,

    /// Emitted when a window is pinned or unpinned. Contains window address and
    /// pin state.
    Pin {
        window_address: String,
        pin_state: bool,
    },

    /// Emitted when an external taskbar-like app requests a window to be
    /// minimized. `minimized` is 0 (unminimized) or 1 (minimized).
    Minimized {
        window_address: String,
        minimized: bool,
    },

    /// Emitted when an app requests to ring the system bell via
    /// `xdg-system-bell-v1`. `window_address` may be empty.
    Bell { window_address: String },
}

#[derive(Error, Debug)]
pub enum ParseEventError {
    #[error("missing '>>' delimiter in input")]
    MissingDelimiter,
    #[error(
        "invalid field count for event '{event}': expected {expected}, got {got}"
    )]
    InvalidFieldCount {
        event: String,
        expected: usize,
        got: usize,
    },
    #[error("missing required field '{field}' in event '{event}'")]
    MissingField { event: String, field: String },
    #[error("invalid boolean value: {0}")]
    InvalidBool(String),
    #[error("invalid integer '{value}': {source}")]
    InvalidInt {
        value: String,
        source: std::num::ParseIntError,
    },
    #[error("unknown event: {0}")]
    UnknownEvent(String),
}

fn field_count_err(
    event: &str,
    expected: usize,
    got: usize,
) -> ParseEventError {
    ParseEventError::InvalidFieldCount {
        event: event.to_string(),
        expected,
        got,
    }
}

fn missing_field(event: &str, field: &str) -> ParseEventError {
    ParseEventError::MissingField {
        event: event.to_string(),
        field: field.to_string(),
    }
}

fn parse_bool(s: &str) -> Result<bool, ParseEventError> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(ParseEventError::InvalidBool(s.to_string())),
    }
}

fn parse_i32(s: &str) -> Result<i32, ParseEventError> {
    s.parse::<i32>().map_err(|e| ParseEventError::InvalidInt {
        value: s.to_string(),
        source: e,
    })
}

impl FromStr for HyprEvent {
    type Err = ParseEventError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (event_name, data) = s
            .split_once(">>")
            .ok_or(ParseEventError::MissingDelimiter)?;

        let parts: Vec<&str> = if data.is_empty() {
            Vec::new()
        } else {
            data.split(',').collect()
        };

        match event_name {
            "workspace" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::Workspace {
                    workspace_name: parts[0].to_string(),
                })
            }
            "workspacev2" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::WorkspaceV2 {
                    id: parse_i32(parts[0])?,
                    name: parts[1].to_string(),
                })
            }
            "focusedmon" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::FocusedMon {
                    monitor_name: parts[0].to_string(),
                    workspace_name: parts[1].to_string(),
                })
            }
            "focusedmonv2" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::FocusedMonV2 {
                    monitor_connector: parts[0].to_string(),
                    workspace_id: parse_i32(parts[1])?,
                })
            }
            "activewindow" => {
                if data.is_empty() {
                    return Err(missing_field(event_name, "window data"));
                }
                let (class, title) = data.split_once(',').unwrap_or((data, ""));
                Ok(HyprEvent::ActiveWindow {
                    window_class: class.to_string(),
                    window_title: title.to_string(),
                })
            }
            "activewindowv2" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::ActiveWindowV2 {
                    window_address: parts[0].to_string(),
                })
            }
            "fullscreen" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::Fullscreen {
                    state: parse_bool(parts[0])?,
                })
            }
            "monitorremoved" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::MonitorRemoved {
                    name: parts[0].to_string(),
                })
            }
            "monitorremovedv2" => {
                let mut split = data.splitn(3, ',');
                let id = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "id"))?;
                let name = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "name"))?;
                let description = split.next().unwrap_or("");
                Ok(HyprEvent::MonitorRemovedV2 {
                    id: parse_i32(id)?,
                    name: name.to_string(),
                    description: description.to_string(),
                })
            }
            "monitoradded" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::MonitorAdded {
                    name: parts[0].to_string(),
                })
            }
            "monitoraddedv2" => {
                let mut split = data.splitn(3, ',');
                let id = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "id"))?;
                let name = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "name"))?;
                let description = split.next().unwrap_or("");
                Ok(HyprEvent::MonitorAddedV2 {
                    id: parse_i32(id)?,
                    name: name.to_string(),
                    description: description.to_string(),
                })
            }
            "createworkspace" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::CreateWorkspace {
                    name: parts[0].to_string(),
                })
            }
            "createworkspacev2" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::CreateWorkspaceV2 {
                    id: parse_i32(parts[0])?,
                    name: parts[1].to_string(),
                })
            }
            "destroyworkspace" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::DestroyWorkspace {
                    name: parts[0].to_string(),
                })
            }
            "destroyworkspacev2" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::DestroyWorkspaceV2 {
                    id: parse_i32(parts[0])?,
                    name: parts[1].to_string(),
                })
            }
            "moveworkspace" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::MoveWorkspace {
                    workspace_name: parts[0].to_string(),
                    monitor_name: parts[1].to_string(),
                })
            }
            "moveworkspacev2" => {
                if parts.len() < 3 {
                    return Err(field_count_err(event_name, 3, parts.len()));
                }
                Ok(HyprEvent::MoveWorkspaceV2 {
                    id: parse_i32(parts[0])?,
                    workspace_name: parts[1].to_string(),
                    monitor_name: parts[2].to_string(),
                })
            }
            "renameworkspace" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::RenameWorkspace {
                    id: parse_i32(parts[0])?,
                    new_name: parts[1].to_string(),
                })
            }
            "activespecial" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::ActiveSpecial {
                    workspace_name: parts[0].to_string(),
                    monitor_name: parts[1].to_string(),
                })
            }
            "activespecialv2" => {
                if parts.len() < 3 {
                    return Err(field_count_err(event_name, 3, parts.len()));
                }
                Ok(HyprEvent::ActiveSpecialV2 {
                    id: parse_i32(parts[0])?,
                    workspace_name: parts[1].to_string(),
                    monitor_name: parts[2].to_string(),
                })
            }
            "activelayout" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::ActiveLayout {
                    keyboard_name: parts[0].to_string(),
                    layout_name: parts[1].to_string(),
                })
            }
            "openwindow" => {
                let mut split = data.splitn(4, ',');
                let address = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "address"))?;
                let ws_name = split.next().ok_or_else(|| {
                    missing_field(event_name, "workspace_name")
                })?;
                let class = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "class"))?;
                let title = split.next().unwrap_or("");
                Ok(HyprEvent::OpenWindow {
                    window_address: address.to_string(),
                    workspace_name: ws_name.to_string(),
                    window_class: class.to_string(),
                    window_title: title.to_string(),
                })
            }
            "closewindow" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::CloseWindow {
                    window_address: parts[0].to_string(),
                })
            }
            "kill" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::Kill {
                    window_address: parts[0].to_string(),
                })
            }
            "movewindow" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::MoveWindow {
                    window_address: parts[0].to_string(),
                    workspace_name: parts[1].to_string(),
                })
            }
            "movewindowv2" => {
                if parts.len() < 3 {
                    return Err(field_count_err(event_name, 3, parts.len()));
                }
                Ok(HyprEvent::MoveWindowV2 {
                    window_address: parts[0].to_string(),
                    workspace_id: parse_i32(parts[1])?,
                    workspace_name: parts[2].to_string(),
                })
            }
            "openlayer" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::OpenLayer {
                    namespace: parts[0].to_string(),
                })
            }
            "closelayer" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::CloseLayer {
                    namespace: parts[0].to_string(),
                })
            }
            "submap" => Ok(HyprEvent::Submap {
                submap_name: data.to_string(),
            }),
            "changefloatingmode" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::ChangeFloatingMode {
                    window_address: parts[0].to_string(),
                    floating: parse_bool(parts[1])?,
                })
            }
            "urgent" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::Urgent {
                    window_address: parts[0].to_string(),
                })
            }
            "screencast" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::Screencast {
                    state: parse_bool(parts[0])?,
                    owner: parts[1].to_string(),
                })
            }
            "screencastv2" => {
                let mut split = data.splitn(3, ',');
                let state = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "state"))?;
                let owner = split
                    .next()
                    .ok_or_else(|| missing_field(event_name, "owner"))?;
                let name = split.next().unwrap_or("");
                Ok(HyprEvent::ScreencastV2 {
                    state: parse_bool(state)?,
                    owner: owner.to_string(),
                    name: name.to_string(),
                })
            }
            "windowtitle" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::WindowTitle {
                    window_address: parts[0].to_string(),
                })
            }
            "windowtitlev2" => {
                if data.is_empty() {
                    return Err(missing_field(event_name, "window_title data"));
                }
                let (addr, title) = data.split_once(',').unwrap_or((data, ""));
                Ok(HyprEvent::WindowTitleV2 {
                    window_address: addr.to_string(),
                    window_title: title.to_string(),
                })
            }
            "togglegroup" => {
                let (state_str, handles) =
                    data.split_once(',').unwrap_or((data, ""));
                Ok(HyprEvent::ToggleGroup {
                    state: parse_bool(state_str)?,
                    handles: handles.to_string(),
                })
            }
            "moveintogroup" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::MoveIntoGroup {
                    window_address: parts[0].to_string(),
                })
            }
            "moveoutofgroup" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::MoveOutOfGroup {
                    window_address: parts[0].to_string(),
                })
            }
            "ignoregrouplock" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::IgnoreGroupLock {
                    state: parse_bool(parts[0])?,
                })
            }
            "lockgroups" => {
                if parts.len() != 1 {
                    return Err(field_count_err(event_name, 1, parts.len()));
                }
                Ok(HyprEvent::LockGroups {
                    state: parse_bool(parts[0])?,
                })
            }
            "configreloaded" => Ok(HyprEvent::ConfigReloaded),
            "pin" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::Pin {
                    window_address: parts[0].to_string(),
                    pin_state: parse_bool(parts[1])?,
                })
            }
            "minimized" => {
                if parts.len() != 2 {
                    return Err(field_count_err(event_name, 2, parts.len()));
                }
                Ok(HyprEvent::Minimized {
                    window_address: parts[0].to_string(),
                    minimized: parse_bool(parts[1])?,
                })
            }
            "bell" => {
                let addr =
                    parts.get(0).map(|s| s.to_string()).unwrap_or_default();
                Ok(HyprEvent::Bell {
                    window_address: addr,
                })
            }
            _ => Err(ParseEventError::UnknownEvent(event_name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // Positive
    #[test]
    fn parse_workspace() {
        let event = HyprEvent::from_str("workspace>>2").unwrap();
        assert!(
            matches!(event, HyprEvent::Workspace { workspace_name } if workspace_name == "2")
        );
    }

    #[test]
    fn parse_workspacev2() {
        let event = HyprEvent::from_str("workspacev2>>2,2").unwrap();
        assert!(
            matches!(event, HyprEvent::WorkspaceV2 { id: 2, name } if name == "2")
        );
    }

    #[test]
    fn parse_focusedmon() {
        let event = HyprEvent::from_str("focusedmon>>DP-1,1").unwrap();
        assert!(
            matches!(event, HyprEvent::FocusedMon { monitor_name, workspace_name }
            if monitor_name == "DP-1" && workspace_name == "1")
        );
    }

    #[test]
    fn parse_focusedmonv2() {
        let event = HyprEvent::from_str("focusedmonv2>>DP-1,1").unwrap();
        assert!(
            matches!(event, HyprEvent::FocusedMonV2 { monitor_connector, workspace_id: 1 }
            if monitor_connector == "DP-1")
        );
    }

    #[test]
    fn parse_activewindow_simple() {
        let event = HyprEvent::from_str("activewindow>>kitty,~").unwrap();
        assert!(
            matches!(event, HyprEvent::ActiveWindow { window_class, window_title }
            if window_class == "kitty" && window_title == "~")
        );
    }

    #[test]
    fn parse_activewindow_title_with_comma() {
        // Заголовок окна может содержать запятые
        let event =
            HyprEvent::from_str("activewindow>>google-chrome,Some title")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::ActiveWindow { window_class, window_title }
            if window_class == "google-chrome"
            && window_title == "Some title")
        );
    }

    #[test]
    fn parse_activewindowv2() {
        let event =
            HyprEvent::from_str("activewindowv2>>5609ae16f400").unwrap();
        assert!(matches!(event, HyprEvent::ActiveWindowV2 { window_address }
            if window_address == "5609ae16f400"));
    }

    #[test]
    fn parse_fullscreen_enter() {
        let event = HyprEvent::from_str("fullscreen>>1").unwrap();
        assert!(matches!(event, HyprEvent::Fullscreen { state: true }));
    }

    #[test]
    fn parse_fullscreen_exit() {
        let event = HyprEvent::from_str("fullscreen>>0").unwrap();
        assert!(matches!(event, HyprEvent::Fullscreen { state: false }));
    }

    #[test]
    fn parse_monitorremoved() {
        let event = HyprEvent::from_str("monitorremoved>>DP-2").unwrap();
        assert!(
            matches!(event, HyprEvent::MonitorRemoved { name } if name == "DP-2")
        );
    }

    #[test]
    fn parse_monitorremovedv2() {
        let event = HyprEvent::from_str(
            "monitorremovedv2>>0,DP-2,Some description, with comma",
        )
        .unwrap();
        assert!(
            matches!(event, HyprEvent::MonitorRemovedV2 { id: 0, name, description }
            if name == "DP-2" && description == "Some description, with comma")
        );
    }

    #[test]
    fn parse_monitoradded() {
        let event = HyprEvent::from_str("monitoradded>>eDP-1").unwrap();
        assert!(
            matches!(event, HyprEvent::MonitorAdded { name } if name == "eDP-1")
        );
    }

    #[test]
    fn parse_monitoraddedv2() {
        let event =
            HyprEvent::from_str("monitoraddedv2>>1,eDP-1,Laptop Screen")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::MonitorAddedV2 { id: 1, name, description }
            if name == "eDP-1" && description == "Laptop Screen")
        );
    }

    #[test]
    fn parse_createworkspace() {
        let event = HyprEvent::from_str("createworkspace>>5").unwrap();
        assert!(
            matches!(event, HyprEvent::CreateWorkspace { name } if name == "5")
        );
    }

    #[test]
    fn parse_createworkspacev2() {
        let event = HyprEvent::from_str("createworkspacev2>>5,5").unwrap();
        assert!(
            matches!(event, HyprEvent::CreateWorkspaceV2 { id: 5, name } if name == "5")
        );
    }

    #[test]
    fn parse_destroyworkspace() {
        let event = HyprEvent::from_str("destroyworkspace>>3").unwrap();
        assert!(
            matches!(event, HyprEvent::DestroyWorkspace { name } if name == "3")
        );
    }

    #[test]
    fn parse_destroyworkspacev2() {
        let event = HyprEvent::from_str("destroyworkspacev2>>3,3").unwrap();
        assert!(
            matches!(event, HyprEvent::DestroyWorkspaceV2 { id: 3, name } if name == "3")
        );
    }

    #[test]
    fn parse_moveworkspace() {
        let event = HyprEvent::from_str("moveworkspace>>2,DP-1").unwrap();
        assert!(
            matches!(event, HyprEvent::MoveWorkspace { workspace_name, monitor_name }
            if workspace_name == "2" && monitor_name == "DP-1")
        );
    }

    #[test]
    fn parse_moveworkspacev2() {
        let event = HyprEvent::from_str("moveworkspacev2>>2,2,DP-1").unwrap();
        assert!(
            matches!(event, HyprEvent::MoveWorkspaceV2 { id: 2, workspace_name, monitor_name }
            if workspace_name == "2" && monitor_name == "DP-1")
        );
    }

    #[test]
    fn parse_renameworkspace() {
        let event =
            HyprEvent::from_str("renameworkspace>>3,новое имя").unwrap();
        assert!(
            matches!(event, HyprEvent::RenameWorkspace { id: 3, new_name }
            if new_name == "новое имя")
        );
    }

    #[test]
    fn parse_activespecial() {
        let event =
            HyprEvent::from_str("activespecial>>special:magic,DP-1").unwrap();
        assert!(
            matches!(event, HyprEvent::ActiveSpecial { workspace_name, monitor_name }
            if workspace_name == "special:magic" && monitor_name == "DP-1")
        );
    }

    #[test]
    fn parse_activespecialv2() {
        let event =
            HyprEvent::from_str("activespecialv2>>42,special:magic,DP-1")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::ActiveSpecialV2 { id: 42, workspace_name, monitor_name }
            if workspace_name == "special:magic" && monitor_name == "DP-1")
        );
    }

    #[test]
    fn parse_activelayout() {
        let event =
            HyprEvent::from_str("activelayout>>gaming-keyboard,Russian")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::ActiveLayout { keyboard_name, layout_name }
            if keyboard_name == "gaming-keyboard" && layout_name == "Russian")
        );
    }

    #[test]
    fn parse_openwindow() {
        let event =
            HyprEvent::from_str("openwindow>>5609ae256fc0,1,kitty,kitty")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::OpenWindow { window_address, workspace_name, window_class, window_title }
            if window_address == "5609ae256fc0"
            && workspace_name == "1"
            && window_class == "kitty"
            && window_title == "kitty")
        );
    }

    #[test]
    fn parse_closewindow() {
        let event = HyprEvent::from_str("closewindow>>5609ae256fc0").unwrap();
        assert!(
            matches!(event, HyprEvent::CloseWindow { window_address } if window_address == "5609ae256fc0")
        );
    }

    #[test]
    fn parse_kill() {
        let event = HyprEvent::from_str("kill>>deadbeef").unwrap();
        assert!(
            matches!(event, HyprEvent::Kill { window_address } if window_address == "deadbeef")
        );
    }

    #[test]
    fn parse_movewindow() {
        let event = HyprEvent::from_str("movewindow>>5609ae16f400,2").unwrap();
        assert!(
            matches!(event, HyprEvent::MoveWindow { window_address, workspace_name }
            if window_address == "5609ae16f400" && workspace_name == "2")
        );
    }

    #[test]
    fn parse_movewindowv2() {
        let event =
            HyprEvent::from_str("movewindowv2>>5609ae16f400,2,myworkspace")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::MoveWindowV2 { window_address, workspace_id: 2, workspace_name }
            if window_address == "5609ae16f400" && workspace_name == "myworkspace")
        );
    }

    #[test]
    fn parse_openlayer() {
        let event = HyprEvent::from_str("openlayer>>gtk-layer-shell").unwrap();
        assert!(
            matches!(event, HyprEvent::OpenLayer { namespace } if namespace == "gtk-layer-shell")
        );
    }

    #[test]
    fn parse_closelayer() {
        let event = HyprEvent::from_str("closelayer>>waybar").unwrap();
        assert!(
            matches!(event, HyprEvent::CloseLayer { namespace } if namespace == "waybar")
        );
    }

    #[test]
    fn parse_submap_empty() {
        let event = HyprEvent::from_str("submap>>").unwrap();
        assert!(
            matches!(event, HyprEvent::Submap { submap_name } if submap_name.is_empty())
        );
    }

    #[test]
    fn parse_submap_value() {
        let event = HyprEvent::from_str("submap>>mysubmap").unwrap();
        assert!(
            matches!(event, HyprEvent::Submap { submap_name } if submap_name == "mysubmap")
        );
    }

    #[test]
    fn parse_changefloatingmode() {
        let event = HyprEvent::from_str("changefloatingmode>>0x123,1").unwrap();
        assert!(
            matches!(event, HyprEvent::ChangeFloatingMode { window_address, floating: true }
            if window_address == "0x123")
        );
    }

    #[test]
    fn parse_urgent() {
        let event = HyprEvent::from_str("urgent>>0xABC").unwrap();
        assert!(
            matches!(event, HyprEvent::Urgent { window_address } if window_address == "0xABC")
        );
    }

    #[test]
    fn parse_screencast() {
        let event = HyprEvent::from_str("screencast>>1,DP-1").unwrap();
        assert!(
            matches!(event, HyprEvent::Screencast { state: true, owner } if owner == "DP-1")
        );
    }

    #[test]
    fn parse_screencastv2_with_name_containing_comma() {
        let event =
            HyprEvent::from_str("screencastv2>>0,window,My Window, with comma")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::ScreencastV2 { state: false, owner, name }
            if owner == "window" && name == "My Window, with comma")
        );
    }

    #[test]
    fn parse_windowtitle() {
        let event = HyprEvent::from_str("windowtitle>>0xdeadbeef").unwrap();
        assert!(
            matches!(event, HyprEvent::WindowTitle { window_address } if window_address == "0xdeadbeef")
        );
    }

    #[test]
    fn parse_windowtitlev2_with_title() {
        let event =
            HyprEvent::from_str("windowtitlev2>>0xdeadbeef,My Window Title")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::WindowTitleV2 { window_address, window_title }
            if window_address == "0xdeadbeef" && window_title == "My Window Title")
        );
    }

    #[test]
    fn parse_windowtitlev2_empty_title() {
        let event = HyprEvent::from_str("windowtitlev2>>0xdeadbeef,").unwrap();
        assert!(
            matches!(event, HyprEvent::WindowTitleV2 { window_address, window_title }
            if window_address == "0xdeadbeef" && window_title.is_empty())
        );
    }

    #[test]
    fn parse_togglegroup_with_handles() {
        let event =
            HyprEvent::from_str("togglegroup>>0,64cea2525760,64cea2522380")
                .unwrap();
        assert!(
            matches!(event, HyprEvent::ToggleGroup { state: false, handles }
            if handles == "64cea2525760,64cea2522380")
        );
    }

    #[test]
    fn parse_moveintogroup() {
        let event = HyprEvent::from_str("moveintogroup>>0x123").unwrap();
        assert!(
            matches!(event, HyprEvent::MoveIntoGroup { window_address } if window_address == "0x123")
        );
    }

    #[test]
    fn parse_moveoutofgroup() {
        let event = HyprEvent::from_str("moveoutofgroup>>0x456").unwrap();
        assert!(
            matches!(event, HyprEvent::MoveOutOfGroup { window_address } if window_address == "0x456")
        );
    }

    #[test]
    fn parse_ignoregrouplock() {
        let event = HyprEvent::from_str("ignoregrouplock>>1").unwrap();
        assert!(matches!(event, HyprEvent::IgnoreGroupLock { state: true }));
    }

    #[test]
    fn parse_lockgroups() {
        let event = HyprEvent::from_str("lockgroups>>0").unwrap();
        assert!(matches!(event, HyprEvent::LockGroups { state: false }));
    }

    #[test]
    fn parse_configreloaded() {
        let event = HyprEvent::from_str("configreloaded>>").unwrap();
        assert!(matches!(event, HyprEvent::ConfigReloaded));
    }

    #[test]
    fn parse_pin() {
        let event = HyprEvent::from_str("pin>>0x789,1").unwrap();
        assert!(
            matches!(event, HyprEvent::Pin { window_address, pin_state: true }
            if window_address == "0x789")
        );
    }

    #[test]
    fn parse_minimized() {
        let event = HyprEvent::from_str("minimized>>0xabc,0").unwrap();
        assert!(
            matches!(event, HyprEvent::Minimized { window_address, minimized: false }
            if window_address == "0xabc")
        );
    }

    #[test]
    fn parse_bell_with_address() {
        let event = HyprEvent::from_str("bell>>0x123").unwrap();
        assert!(
            matches!(event, HyprEvent::Bell { window_address } if window_address == "0x123")
        );
    }

    #[test]
    fn parse_bell_empty() {
        let event = HyprEvent::from_str("bell>>").unwrap();
        assert!(
            matches!(event, HyprEvent::Bell { window_address } if window_address.is_empty())
        );
    }

    // Negative
    #[test]
    fn error_missing_delimiter() {
        let err = HyprEvent::from_str("workspace").unwrap_err();
        assert!(matches!(err, ParseEventError::MissingDelimiter));
    }

    #[test]
    fn error_invalid_field_count() {
        let err = HyprEvent::from_str("workspacev2>>1").unwrap_err();
        assert!(
            matches!(err, ParseEventError::InvalidFieldCount { event, expected: 2, got: 1 }
            if event == "workspacev2")
        );
    }

    #[test]
    fn error_invalid_bool() {
        let err = HyprEvent::from_str("fullscreen>>2").unwrap_err();
        assert!(matches!(err, ParseEventError::InvalidBool(v) if v == "2"));
    }

    #[test]
    fn error_invalid_int() {
        let err = HyprEvent::from_str("workspacev2>>abc,2").unwrap_err();
        assert!(
            matches!(err, ParseEventError::InvalidInt { value, .. } if value == "abc")
        );
    }

    #[test]
    fn error_unknown_event() {
        let err = HyprEvent::from_str("nonexistent>>data").unwrap_err();
        assert!(
            matches!(err, ParseEventError::UnknownEvent(e) if e == "nonexistent")
        );
    }

    #[test]
    fn error_missing_field_in_monitorremovedv2() {
        let err = HyprEvent::from_str("monitorremovedv2>>0").unwrap_err();
        assert!(matches!(err, ParseEventError::MissingField { event, field }
            if event == "monitorremovedv2" && field == "name"));
    }

    #[test]
    fn error_missing_field_in_openwindow() {
        let err = HyprEvent::from_str("openwindow>>addr").unwrap_err();
        assert!(matches!(err, ParseEventError::MissingField { event, field }
            if event == "openwindow" && field == "workspace_name"));
    }

    #[test]
    fn error_activewindow_empty_data() {
        let err = HyprEvent::from_str("activewindow>>").unwrap_err();
        assert!(matches!(err, ParseEventError::MissingField { event, field }
            if event == "activewindow" && field == "window data"));
    }

    #[test]
    fn error_windowtitlev2_empty_data() {
        let err = HyprEvent::from_str("windowtitlev2>>").unwrap_err();
        assert!(matches!(err, ParseEventError::MissingField { event, field }
            if event == "windowtitlev2" && field == "window_title data"));
    }
}
