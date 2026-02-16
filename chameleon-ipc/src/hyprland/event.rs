use anyhow::{anyhow, bail};
use std::{
    fmt::{self, Debug},
    str::FromStr,
};

/// (&str, HyprEvent variant, variant fields) -> HyprEvent
macro_rules! event {
    ($data:expr, $event:ident, $($field:ident),+) => {{
        let mut iter = $data.split(',');

        HyprEvent::$event {
            $(
                $field: iter.next().unwrap().parse().unwrap(),
            )+
        }
    }};
}

/// Contains all Hyprland events
#[derive(Debug, Clone, Default)]
pub enum HyprEvent {
    ActiveLayout {
        keyboard_name: String,
        layout_name: String,
    },
    WorkspaceV2 {
        id: i32,
        name: String,
    },
    FocusedMonV2 {
        monitor_connector: String,
        workspace_id: i32,
    },
    MonitorRemoved {
        name: String,
    },
    MonitorAdded {
        name: String,
    },
    CreateWorkspaceV2 {
        id: i32,
        name: String,
    },
    DestroyWorkspaceV2 {
        id: i32,
        name: String,
    },
    #[default]
    None,
}

impl FromStr for HyprEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (prefix, data) =
            s.split_once(">>").ok_or_else(|| anyhow!("invalid input"))?;

        Ok(match prefix {
            "activelayout" => {
                event!(data, ActiveLayout, keyboard_name, layout_name)
            }
            "workspacev2" => event!(data, WorkspaceV2, id, name),
            "focusedmonv2" => {
                event!(data, FocusedMonV2, monitor_connector, workspace_id)
            }
            "createworkspacev2" => event!(data, CreateWorkspaceV2, id, name),
            "destroyworkspacev2" => event!(data, DestroyWorkspaceV2, id, name),
            _ => bail!("Undefined hyprland event: '{}'", prefix),
        })
    }
}

impl fmt::Display for HyprEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
