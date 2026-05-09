use crate::common::Metadata;
use crate::modules::{BaseModule, ModuleFactory};
use anyhow::Result;
use chameleon_ipc::compositor::Compositor;
use chameleon_ipc::hyprland::HyprEvent;
use chameleon_ipc::{COMPOSITOR, CompositorVariant};
use gtk::glib::clone;
use gtk::prelude::WidgetExt;
use gtke::Component;
use gtkio::future::{spawn, spawn_with_local_callback};
use std::rc::Rc;
use std::sync::LazyLock;
use tokio::sync::watch;

pub static LAYOUT_SENDER: LazyLock<watch::Sender<String>> =
    LazyLock::new(|| {
        let watch_sender = watch::Sender::new(String::new());
        spawn(KeyboardLayout::background_task(watch_sender.clone()));

        if let CompositorVariant::Hyprland(hyprland) = &*COMPOSITOR {
            spawn_with_local_callback(
                async move {
                    let devices = hyprland.devices().await.unwrap();
                    let active_keymap = devices
                        .keyboards
                        .into_iter()
                        .find(|kb| kb.main)
                        .unwrap()
                        .active_keymap;

                    let short_name =
                        KeyboardLayout::shrink_layout_name(&active_keymap);

                    short_name
                },
                clone!(
                    #[strong]
                    watch_sender,
                    move |active_keymap| {
                        if let Err(e) = watch_sender.send(active_keymap) {
                            log::error!("{e}");
                        }
                    }
                ),
            );
        }

        watch_sender
    });

#[derive(Debug, Component)]
pub struct KeyboardLayout {
    #[root]
    base: BaseModule,
}

impl ModuleFactory for KeyboardLayout {
    type Config = ();

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> Result<Rc<dyn Component>> {
        let kb_layout = KeyboardLayout::new();

        Ok(Rc::new(kb_layout))
    }
}

impl KeyboardLayout {
    const NAME: &str = "keyboard-layout";

    pub fn new() -> Self {
        let state = LAYOUT_SENDER.subscribe();
        let base = BaseModule::new(state);

        base.set_widget_name(Self::NAME);
        base.add_css_class("module");

        Self { base }
    }

    pub fn shrink_layout_name(name: &String) -> String {
        name.chars()
            .take(2)
            .flat_map(|c| c.to_uppercase())
            .collect()
    }

    async fn background_task(sender: watch::Sender<String>) {
        if let CompositorVariant::Hyprland(hyprland) = &*COMPOSITOR {
            let mut receiver = hyprland.subscribe();

            loop {
                if let Ok(event) = receiver.recv().await
                    && let HyprEvent::ActiveLayout { layout_name, .. } = event
                {
                    let short_name = Self::shrink_layout_name(&layout_name);

                    sender.send(short_name).unwrap();
                }
            }
        }
    }
}
