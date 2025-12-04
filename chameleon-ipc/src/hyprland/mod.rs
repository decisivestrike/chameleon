pub mod events;
pub mod workspace;
pub use events::HyprEvent;

use anyhow::Result;
use grapes::tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use std::{env::var, sync::LazyLock};

pub static XDG_RUNTIME_DIR: LazyLock<String> = LazyLock::new(|| {
    var("XDG_RUNTIME_DIR")
        .expect("XDG_RUNTIME_DIR not set — not in a desktop session?")
});

/// Hyprland instance signature
pub static HIS: LazyLock<String> = LazyLock::new(|| {
    var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("Not running inside a Hyprland session")
});

/// Socket for listening events
///
/// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
pub static TX_SOCK: LazyLock<String> = LazyLock::new(|| {
    format!("{}/hypr/{}/.socket2.sock", *XDG_RUNTIME_DIR, *HIS)
});

/// Socket for sending commands
///
/// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
pub static RX_SOCK: LazyLock<String> = LazyLock::new(|| {
    format!("{}/hypr/{}/.socket.sock", *XDG_RUNTIME_DIR, *HIS)
});

pub async fn query(request: &[u8]) -> Result<String> {
    let mut stream = UnixStream::connect(&*RX_SOCK).await?;
    stream.write_all(request).await?;

    let mut data = String::with_capacity(512);
    stream.read_to_string(&mut data).await?;

    Ok(data)
}

pub async fn command(command: &[u8]) -> Result<()> {
    let mut stream = UnixStream::connect(&*RX_SOCK).await?;
    stream.write_all(command).await?;

    Ok(())
}
