use anyhow::{anyhow, bail};
use grapes::{
    service,
    tokio::{
        io::{AsyncBufReadExt, BufReader},
        net::UnixStream,
        time::sleep,
    },
};
use log::{error, info, warn};
use std::{str::FromStr, time::Duration};

use crate::hyprland::TX_SOCK;

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
#[derive(Debug, Clone)]
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
        monitor_name: String,
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
                event!(data, FocusedMonV2, monitor_name, workspace_id)
            }
            "createworkspacev2" => event!(data, CreateWorkspaceV2, id, name),
            "destroyworkspacev2" => event!(data, DestroyWorkspaceV2, id, name),
            _ => bail!("Undefined hyprland event: '{}'", prefix),
        })
    }
}

service!(HyprlandService -> HyprEvent, async |tx| {
    let backoff = Duration::from_millis(200);

    loop {
        let stream = match UnixStream::connect(&*TX_SOCK).await {
            Ok(s) => {
                info!("connected to {:?}", TX_SOCK);
                s
            }
            Err(e) => {
                error!("connect {:?} failed: {e}", TX_SOCK);
                sleep(backoff).await;
                continue;
            }
        };

        let mut lines = BufReader::new(stream).lines();

        loop {
            let line = match lines.next_line().await {
                Ok(Some(l)) => l,
                Ok(None) => {
                    warn!("event stream EOF; reconnecting…");
                    break;
                }
                Err(e) => {
                    warn!("read line failed: {e}");
                    break;
                }
            };

            let event = match line.parse::<HyprEvent>() {
                Ok(ev) => ev,
                Err(e) => {
                    warn!("parse HyprEvent failed: {e}; line={line}");
                    continue; // пропускаем строку
                }
            };

            if let Err(e) = tx.send(event) {
                warn!("broadcast closed (no receivers): {e}");
                return;
            }
        }

        sleep(backoff).await;
    }
});
