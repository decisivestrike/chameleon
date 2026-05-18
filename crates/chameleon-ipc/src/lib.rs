pub mod compositor;
pub mod hyprland;
pub mod niri;

use crate::compositor::Compositor;
use crate::hyprland::{HyprEvent, Hyprland};
use crate::niri::Niri;
use gtkio::RUNTIME;
use gtkio::workers::WatchWorker;
use niri_ipc::socket::SOCKET_PATH_ENV;
use std::env::var;
use std::sync::LazyLock;
use tracing::error;

pub static COMPOSITOR: LazyLock<CompositorVariant> =
    LazyLock::new(CompositorVariant::define);

pub enum CompositorVariant {
    Hyprland(Hyprland),
    Niri(Niri),
    Unknown,
}

impl CompositorVariant {
    /// Tries to determine which compositor you are using
    fn define() -> Self {
        if let Ok(his) = var("HYPRLAND_INSTANCE_SIGNATURE") {
            let hyprland = Hyprland::init(his);
            CompositorVariant::Hyprland(hyprland)
        } else if let Ok(_) = var(SOCKET_PATH_ENV) {
            let niri = Niri::init().unwrap();
            CompositorVariant::Niri(niri)
        } else {
            CompositorVariant::Unknown
        }
    }
}

pub static WORKSPACES_EVENTS: LazyLock<()> = LazyLock::new(|| {});

pub static KEYBOARD_LAYOUT: LazyLock<WatchWorker<String>> =
    LazyLock::new(|| match &*COMPOSITOR {
        CompositorVariant::Hyprland(hyprland) => {
            let devices = RUNTIME.block_on(hyprland.devices()).unwrap();
            let active_keymap = devices
                .keyboards
                .into_iter()
                .find(|kb| kb.main)
                .unwrap()
                .active_keymap;

            WatchWorker::new(active_keymap, {
                let mut receiver = hyprland.subscribe();
                async move |sender| {
                    loop {
                        if let Ok(event) = receiver.recv().await
                            && let HyprEvent::ActiveLayout {
                                layout_name, ..
                            } = event
                        {
                            if let Err(e) = sender.send(layout_name) {
                                error!("{}", e);
                            }
                        }
                    }
                }
            })
        }
        CompositorVariant::Niri(niri) => unreachable!(),
        CompositorVariant::Unknown => panic!("U should use Hyprland or Niri"),
    });
