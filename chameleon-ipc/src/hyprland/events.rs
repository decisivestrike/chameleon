use crate::hyprland::TX_SOCK;
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
use std::{
    cmp::min,
    fmt::{self, Debug},
    path::Path,
    str::FromStr,
    time::Duration,
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

async fn connect_with_backoff<P>(path: P) -> UnixStream
where
    P: AsRef<Path> + fmt::Display,
{
    let mut delay = Duration::from_millis(100);

    loop {
        match UnixStream::connect(&path).await {
            Ok(stream) => {
                info!("Connected to {path}");
                return stream;
            }
            Err(e) => {
                error!("Connect to {path} failed: {e}");
                sleep(delay).await;

                delay = min(
                    delay + Duration::from_millis(100),
                    Duration::from_secs(1),
                );
            }
        }
    }
}

service!(HyprlandService -> HyprEvent, async |tx| {
    loop {
        let stream = connect_with_backoff(&*TX_SOCK).await;
        let mut lines = BufReader::new(stream).lines();

        loop {
            let line = match lines.next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => {
                    warn!("Event stream EOF; reconnecting...");
                    break;
                }
                Err(e) => {
                    warn!("Read line failed: {e}");
                    break;
                }
            };

            let event = match line.parse::<HyprEvent>() {
                Ok(ev) => ev,
                Err(e) => {
                    warn!("{e}");
                    continue;
                }
            };

            if let Err(e) = tx.send(event) {
                warn!("broadcast closed (no receivers): {e}");
                return;
            }
        }
    }
});
