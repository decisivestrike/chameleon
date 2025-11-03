mod events;
mod workspace;

use anyhow::Result;
use grapes::tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use std::sync::LazyLock;

use crate::core::env_var;

pub static SOCK2_PATH: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}/hypr/{}/.socket2.sock",
        env_var("XDG_RUNTIME_DIR"),
        env_var("HYPRLAND_INSTANCE_SIGNATURE")
    )
});

pub static SOCK_PATH: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}/hypr/{}/.socket.sock",
        env_var("XDG_RUNTIME_DIR"),
        env_var("HYPRLAND_INSTANCE_SIGNATURE")
    )
});

pub async fn query(request: &[u8]) -> Result<String> {
    let mut stream = UnixStream::connect(&*SOCK_PATH).await?;
    stream.write_all(request).await?;

    let mut data = String::with_capacity(512);
    stream.read_to_string(&mut data).await?;

    Ok(data)
}

pub async fn command(command: &[u8]) -> Result<()> {
    let mut stream = UnixStream::connect(&*SOCK_PATH).await?;
    stream.write_all(command).await?;

    Ok(())
}
