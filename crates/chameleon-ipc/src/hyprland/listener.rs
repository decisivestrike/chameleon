use crate::hyprland::{HyprEvent, Hyprland};
use grapes::tokio::io::{AsyncBufReadExt, BufReader};
use grapes::tokio::net::UnixStream;
use grapes::tokio::time::sleep;
use std::cmp::min;
use std::path::PathBuf;
use std::time::Duration;

/// Hyprland event listener
pub struct EventListener;

impl EventListener {
    pub async fn run(hyprland: &'static Hyprland) -> ! {
        let Hyprland {
            event_sender,
            sender_sock,
            ..
        } = hyprland;

        loop {
            let stream = Self::connect_with_backoff(sender_sock).await;
            let mut lines = BufReader::new(stream).lines();

            loop {
                let line = match lines.next_line().await {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        log::warn!("Event stream EOF; reconnecting...");
                        break;
                    }
                    Err(e) => {
                        log::warn!("Read line failed: {e}");
                        break;
                    }
                };

                let event = match line.parse::<HyprEvent>() {
                    Ok(ev) => ev,
                    Err(_e) => {
                        // warn!("{e}");
                        continue;
                    }
                };

                if let Err(e) = event_sender.send(event) {
                    log::warn!("broadcast closed (no receivers): {e}");
                }
            }
        }
    }

    async fn connect_with_backoff(sender_sock: &PathBuf) -> UnixStream {
        let mut delay = Duration::from_millis(100);

        loop {
            match UnixStream::connect(sender_sock).await {
                Ok(stream) => {
                    log::info!("Connected to {}", sender_sock.display());
                    return stream;
                }
                Err(e) => {
                    log::error!(
                        "Connect to {} failed: {e}",
                        sender_sock.display()
                    );

                    sleep(delay).await;

                    delay = min(
                        delay + Duration::from_millis(100),
                        Duration::from_secs(1),
                    );
                }
            }
        }
    }
}
