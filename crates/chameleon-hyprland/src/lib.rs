pub mod event;
pub use event::HyprEvent;
use gtkio::RUNTIME;

pub mod entities;
pub mod socket;

use crate::entities::Workspace;
use crate::entities::device::Devices;
use crate::socket::Socket;
use anyhow::Result;
use std::env::var;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::broadcast::{
    self, Receiver as BroadcastReceiver, Sender as BroadcastSender,
};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct Hyprland {
    /// Socket for sending commands
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
    recv_sock: PathBuf,

    /// Socket for listening events
    ///
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
    sender_sock: Arc<PathBuf>,

    event_listener: RwLock<Option<EventListener>>,
}

impl Hyprland {
    pub fn new(his: String) -> Self {
        let runtime_dir = var("XDG_RUNTIME_DIR")
            .expect("XDG_RUNTIME_DIR not set — not in a desktop session?");

        let recv_sock = format!("{runtime_dir}/hypr/{his}/.socket.sock").into();
        let sender_sock =
            Arc::new(format!("{runtime_dir}/hypr/{his}/.socket2.sock").into());

        Self {
            recv_sock,
            sender_sock,
            event_listener: Default::default(),
        }
    }

    pub async fn connect(&self) -> io::Result<Socket> {
        Socket::connect_to(&*self.sender_sock).await
    }

    pub fn subscribe(&self) -> BroadcastReceiver<HyprEvent> {
        if self.event_listener.read().unwrap().is_none() {
            let mut el = self.event_listener.write().unwrap();
            *el = Some(EventListener::new(self.sender_sock.clone()));
        }

        self.event_listener
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .subscribe()
    }

    /// Issue a lua string to execute dynamically.
    pub async fn eval(&self, lua: &str) -> Result<()> {
        let command = format!("eval {}", lua);

        self.command(command.as_bytes()).await
    }

    /// Dispatch is a shorthand for `eval 'hl.dispatch(...)':`
    ///
    /// Example: `hl.dsp.focus({ workspace = "2" })`
    pub async fn dispatch(&self, dsp: &str) -> Result<()> {
        let command = format!("dispatch {}", dsp);

        self.command(command.as_bytes()).await
    }

    async fn query(&self, request: &[u8]) -> Result<String> {
        let mut stream = UnixStream::connect(&*self.recv_sock).await?;
        stream.write_all(request).await?;

        let mut data = String::new();
        stream.read_to_string(&mut data).await?;

        Ok(data)
    }

    async fn command(&self, command: &[u8]) -> Result<()> {
        let mut stream = UnixStream::connect(&*self.recv_sock).await?;
        stream.write_all(command).await?;

        Ok(())
    }
}

/// Info
impl Hyprland {
    pub async fn workspaces(&self) -> Result<Vec<Workspace>> {
        let json_str = self.query(b"j/workspaces").await?;
        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn active_workspace(&self) -> Result<Workspace> {
        let json_str = self.query(b"j/activeworkspace").await?;
        Ok(serde_json::from_str(&json_str)?)
    }

    pub async fn devices(&self) -> Result<Devices> {
        let json_str = self.query(b"j/devices").await?;
        Ok(serde_json::from_str(&json_str)?)
    }
}

impl Default for Hyprland {
    fn default() -> Self {
        let his = var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        Self::new(his)
    }
}

#[derive(Debug)]
pub struct EventListener {
    _handle: JoinHandle<()>,
    sender: BroadcastSender<HyprEvent>,
}

impl EventListener {
    pub fn new(path: Arc<PathBuf>) -> Self {
        let sender = broadcast::Sender::new(32);

        let _handle = RUNTIME.spawn({
            let sender = sender.clone();
            async move {
                let mut sock = Socket::connect_to(&*path).await.unwrap();

                loop {
                    if let Ok(Some(event)) = sock.wait_event().await {
                        sender.send(event).unwrap();
                    }
                }
            }
        });

        Self { _handle, sender }
    }

    pub fn subscribe(&self) -> BroadcastReceiver<HyprEvent> {
        self.sender.subscribe()
    }
}

// Works inside hyprland only
#[cfg(test)]
mod tests {
    use crate::Hyprland;
    use std::env::var;

    #[tokio::test]
    async fn test_info_active_workspace() {
        let his = var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        let hyprland = Hyprland::new(his);

        let active_workspace = hyprland.active_workspace().await;

        assert!(active_workspace.is_ok())
    }

    #[tokio::test]
    async fn test_info_workspaces() {
        let his = var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        let hyprland = Hyprland::new(his);

        let workspaces = hyprland.workspaces().await;

        assert!(workspaces.is_ok())
    }
}
