use crate::{common::Metadata, modules::ModuleFactory};
use chameleon_config::{CONFIG, panel::ClockConfig};
use chrono::Local;
use grapes::{
    Component, RT, State,
    gtk::{self, Image, prelude::WidgetExt},
    prelude::{BoxExt, containers::GrapesBoxExt},
    state,
    tokio::{
        sync::broadcast::{self, Sender},
        time::sleep,
    },
};
use grapes_components::StatefullLabel;
use std::{rc::Rc, sync::LazyLock, time::Duration};

pub static TIME_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| {
        let sender = broadcast::Sender::new(64);

        RT.spawn(Clock::background_task(&CONFIG.panel.clock, sender.clone()));

        sender
    });

#[derive(Debug, Component)]
pub struct Clock {
    #[root]
    container: gtk::Box,
    _label: StatefullLabel<String>, // panic if remove
}

impl ModuleFactory for Clock {
    type Config = ClockConfig;

    fn create(
        config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        let formatted_time = state(Clock::formatted_time(&config.format));
        formatted_time.track(&TIME_SENDER);

        let clock = Clock::new(&formatted_time);
        Ok(Rc::new(clock))
    }
}

impl Clock {
    const NAME: &str = "clock";

    pub fn new(formatted_time: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(formatted_time);

        let icon = Image::from_icon_name("appointment-symbolic");
        icon.set_size_request(24, 24);

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.append(&icon);
        container.append_ref(&label);

        label.as_ref().set_widget_name(Self::NAME);
        label.as_ref().add_css_class("module");

        Self {
            container,
            _label: label,
        }
    }

    fn formatted_time(format: &str) -> String {
        let time = Local::now();
        time.format(format).to_string()
    }

    async fn background_task(config: &ClockConfig, sender: Sender<String>) {
        let format = &config.format;

        loop {
            let formatted_time = Clock::formatted_time(format);
            sender.send(formatted_time).unwrap();

            sleep(Duration::from_secs(1)).await;
        }
    }
}
