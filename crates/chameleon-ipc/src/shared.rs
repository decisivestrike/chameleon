use crate::{COMPOSITOR, Compositor};
use chameleon_hyprland::HyprEvent;
use gtkio::RUNTIME;
use std::sync::LazyLock;
use tokio::sync::watch;
use tracing::error;

pub static KEYBOARD_LAYOUT: LazyLock<watch::Sender<String>> =
    LazyLock::new(|| match &*COMPOSITOR {
        Compositor::Hyprland(hyprland) => {
            let devices = RUNTIME.block_on(hyprland.devices()).unwrap();
            let active_keymap = devices
                .keyboards
                .into_iter()
                .find(|kb| kb.main)
                .unwrap()
                .active_keymap;

            let sender = watch::Sender::new(active_keymap);

            let sender_clone = sender.clone();
            let mut receiver = hyprland.subscribe();

            RUNTIME.spawn(async move {
                loop {
                    if let Ok(event) = receiver.recv().await
                        && let HyprEvent::ActiveLayout { layout_name, .. } =
                            event
                    {
                        if let Err(e) = sender_clone.send(layout_name) {
                            error!("{}", e);
                        }
                    }
                }
            });

            sender
        }
        Compositor::Unsupported => todo!(),
    });
