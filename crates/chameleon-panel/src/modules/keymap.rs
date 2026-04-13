use crate::common::Metadata;
use crate::modules::ModuleFactory;
use anyhow::Result;
use chameleon_ipc::compositor::Compositor;
use chameleon_ipc::hyprland::HyprEvent;
use chameleon_ipc::{COMPOSITOR, CompositorVariant};
use grapes::prelude::WidgetExt;
use grapes::tokio::sync::broadcast::{self, Sender};
use grapes::tokio::sync::oneshot;
use grapes::{Component, RT, State, glib, state};
use grapes_components::StatefullLabel;
use std::rc::Rc;
use std::sync::LazyLock;

pub static EVENT_LISTENER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| {
        let sender = broadcast::Sender::new(64);

        RT.spawn(Keymap::background_task(sender.clone()));

        sender
    });

#[derive(Debug, Component)]
pub struct Keymap {
    #[root]
    label: StatefullLabel<String>,
}

impl ModuleFactory for Keymap {
    type Config = ();

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> Result<Rc<dyn Component>> {
        if let CompositorVariant::Hyprland(hyprland) = &*COMPOSITOR {
            let (sender, recv) = oneshot::channel();

            let layout = state(String::new());
            layout.track(&*EVENT_LISTENER);

            RT.spawn(async move {
                let devices = hyprland.devices().await.unwrap();
                let active_keymap = devices
                    .keyboards
                    .into_iter()
                    .find(|kb| kb.main)
                    .unwrap()
                    .active_keymap;

                let short_name = Self::shrink_layout_name(&active_keymap);

                if let Err(e) = sender.send(short_name) {
                    log::error!("{e:?}");
                };
            });

            glib::spawn_future_local({
                let state_clone = layout.clone();
                async move {
                    let active_keymap = recv.await.unwrap();
                    state_clone.set(active_keymap)
                }
            });

            let keyboard_layout = Keymap::new(&layout);
            Ok(Rc::new(keyboard_layout))
        } else {
            panic!()
        }
    }
}

impl Keymap {
    const NAME: &str = "keyboard-layout";

    pub fn new(layout: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(layout);

        label.as_ref().set_widget_name(Self::NAME);
        label.as_ref().add_css_class("module");

        Self { label }
    }

    pub fn shrink_layout_name(name: &String) -> String {
        name.chars()
            .take(2)
            .flat_map(|c| c.to_uppercase())
            .collect()
    }

    async fn background_task(sender: Sender<String>) {
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
