pub mod event;
pub use event::HyprEvent;

pub mod entities;
pub mod socket;

use crate::entities::Workspace;
use crate::entities::device::Devices;
use crate::socket::Socket;
use anyhow::Result;
use std::env::var;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::broadcast::{
    self, Receiver as BroadcastReceiver, Sender as BroadcastSender,
};

static EVENT_LISTENER: OnceLock<BroadcastSender<HyprEvent>> = OnceLock::new();

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

    pub async fn connect(&self) -> io::Result<Socket> {
        Socket::connect_to(&self.sender_sock).await
    }

    pub async fn subscribe(&self) -> io::Result<BroadcastReceiver<HyprEvent>> {
        let sender = EVENT_LISTENER.get_or_init(|| {
            let mut sock = tokio::block_on(self.connect())?;
            let sender = broadcast::Sender::new(32);
            let sender_clone = sender.clone();

            tokio::spawn(async move {
                loop {
                    if let Ok(Some(event)) = sock.wait_event().await {
                        sender.send(event).unwrap();
                    }
                }
            });

            sender_clone
        });

        Ok(sender.subscribe())
    }

    /// Issue a lua string to execute dynamically.
    pub async fn eval(&self, lua: &str) -> Result<()> {
        let command = format!("eval {}\0", lua);

        self.command(command.as_bytes()).await
    }

    /// Dispatch is a shorthand for `eval 'hl.dispatch(...)':`
    ///
    /// Example: `hl.dsp.focus({ workspace = "2" })`
    pub async fn dispatch(&self, dsp: &str) -> Result<()> {
        let command = format!("dispatch {}\0", dsp);

        self.command(command.as_bytes()).await
    }

    async fn query(&self, request: &[u8]) -> Result<String> {
        let mut stream = UnixStream::connect(&self.recv_sock).await?;
        stream.write_all(request).await?;

        let mut data = String::new();
        stream.read_to_string(&mut data).await?;

        Ok(data)
    }

    async fn command(&self, command: &[u8]) -> Result<()> {
        let mut stream = UnixStream::connect(&self.recv_sock).await?;
        stream.write_all(command).await?;

        Ok(())
    }
}

/// Info
impl Hyprland {
    pub async fn workspaces(&self) -> Result<Vec<Workspace>> {
        let json_str = self.query(b"j/workspaces\0").await?;
        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn active_workspace(&self) -> Result<Workspace> {
        let json_str = self.query(b"j/activeworkspace\0").await?;
        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn devices(&self) -> Result<Devices> {
        let json_str = self.query(b"j/devices\0").await?;
        Ok(serde_json::from_str(&json_str)?)
    }
}

impl Default for Hyprland {
    fn default() -> Self {
        let his = var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        Self::new(his)
    }
}
