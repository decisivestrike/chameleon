pub mod event;
mod listener;

pub use event::HyprEvent;
pub mod entities;

use crate::entities::Workspace;
use crate::entities::device::Devices;
use crate::listener::EventListener;
use anyhow::Result;
use std::env::var;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Hyprland {
    /// Socket for sending commands
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
    recv_sock: PathBuf,

    /// Socket for listening events
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
    sender_sock: PathBuf,
}

impl Hyprland {
    pub fn new(his: String) -> Self {
        let xdg_runtime_dir = var("XDG_RUNTIME_DIR")
            .expect("XDG_RUNTIME_DIR not set — not in a desktop session?");

        let recv_sock =
            format!("{xdg_runtime_dir}/hypr/{his}/.socket.sock").into();
        let sender_sock =
            format!("{xdg_runtime_dir}/hypr/{his}/.socket2.sock").into();

        Self {
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

    /// Issue a lua string to execute dynamically.
    pub async fn eval(&self, lua: &str) -> Result<()> {
        let command = format!("eval {}\0", lua);

        self.command(command.as_bytes()).await
    }

    /// Calls dispatcher
    /// Dispatch is a shorthand for `eval 'hl.dispatch(...)':`
    ///
    /// Example: `hl.dsp.focus({ workspace = "2" })`
    pub async fn dispatch(&self, dsp: &str) -> Result<()> {
        let command = format!("dispatch {}\0", dsp);

        self.command(command.as_bytes()).await
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
}
