use crate::{common::Metadata, modules::ModuleFactory};
use anyhow::Result;
use chameleon_ipc::{
    COMPOSITOR, CompositorVariant, compositor::Compositor, hyprland::HyprEvent,
};
use grapes::{
    Component, RT, State, glib,
    prelude::WidgetExt,
    state,
    tokio::sync::{
        broadcast::{self, Sender},
        oneshot,
    },
};
use grapes_components::StatefullLabel;
use std::{rc::Rc, sync::LazyLock};

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
        config: &Self::Config,
        meta: &Metadata,
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

                if let Err(e) = sender.send(active_keymap) {
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

    async fn background_task(sender: Sender<String>) {
        if let CompositorVariant::Hyprland(hyprland) = &*COMPOSITOR {
            let mut receiver = hyprland.subscribe();

            loop {
                if let Ok(event) = receiver.recv().await
                    && let HyprEvent::ActiveLayout { layout_name, .. } = event
                {
                    sender.send(layout_name).unwrap();
                }
            }
        }
    }
}
