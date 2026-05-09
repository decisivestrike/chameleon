pub mod event;
mod listener;
pub mod workspace;

pub use event::HyprEvent;
use gtk::gdk::prelude::MonitorExt;
use gtkio::future::spawn;
pub use workspace::*;

mod device;

use crate::compositor::Compositor;
use crate::hyprland::device::Devices;
use crate::hyprland::listener::EventListener;
use crate::{COMPOSITOR, CompositorVariant};
use anyhow::Result;
use gtk::gdk::Monitor;
use std::env::var;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Hyprland {
    event_sender: broadcast::Sender<HyprEvent>,

    /// Socket for sending commands
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
    recv_sock: PathBuf,

    /// Socket for listening events
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
    sender_sock: PathBuf,
}

impl Compositor for Hyprland {
    type Message = HyprEvent;

    fn subscribe(&self) -> broadcast::Receiver<HyprEvent> {
        self.event_sender.subscribe()
    }
}

impl Hyprland {
    pub fn init(his: String) -> Self {
        let xdg_runtime_dir = var("XDG_RUNTIME_DIR")
            .expect("XDG_RUNTIME_DIR not set — not in a desktop session?");

        let event_sender = broadcast::Sender::new(64);
        let recv_sock =
            format!("{xdg_runtime_dir}/hypr/{his}/.socket.sock").into();
        let sender_sock =
            format!("{xdg_runtime_dir}/hypr/{his}/.socket2.sock").into();

        spawn(async move {
            if let CompositorVariant::Hyprland(hyprland) = &*COMPOSITOR {
                spawn(EventListener::run(hyprland));
            }
        });

        Self {
            event_sender,
            recv_sock,
            sender_sock,
        }
    }

    pub async fn query(&self, request: &[u8]) -> Result<String> {
        let mut stream = UnixStream::connect(&self.recv_sock).await?;
        stream.write_all(request).await?;

        let mut data = String::with_capacity(512);
        stream.read_to_string(&mut data).await?;

        Ok(data)
    }

    pub async fn command(&self, command: &[u8]) -> Result<()> {
        let mut stream = UnixStream::connect(&self.recv_sock).await?;
        stream.write_all(command).await?;

        Ok(())
    }

    pub async fn active_workspace(&self) -> Result<Workspace> {
        let json_str = self.query(b"j/activeworkspace\0").await?;

        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn devices(&self) -> Result<Devices> {
        let json_str = self.query(b"j/devices\0").await?;

        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn workspaces(&self) -> Result<Vec<Workspace>> {
        let json_str = self.query(b"j/workspaces\0").await?;

        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn workspaces_on_monitor(
        &self,
        monitor: &Monitor,
    ) -> Result<Vec<Workspace>> {
        Ok(self
            .workspaces()
            .await?
            .into_iter()
            .filter(|w| w.monitor == monitor.connector().unwrap().to_string())
            .collect())
    }
}
