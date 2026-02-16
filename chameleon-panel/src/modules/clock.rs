use crate::{common::Metadata, modules::ModuleFactory};
use chameleon_config::{CONFIG, panel::ClockConfig};
use chrono::Local;
use grapes::{
    Component, RT, State,
    gtk::prelude::WidgetExt,
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
    label: StatefullLabel<String>,
}

impl ModuleFactory for Clock {
    type Config = ClockConfig;

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        let formatted_time = state(Local::now().to_string());
        formatted_time.track(&TIME_SENDER);

        let clock = Clock::new(&formatted_time);
        Ok(Rc::new(clock))
    }
}

impl Clock {
    const NAME: &str = "clock";

    pub fn new(formatted_time: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(formatted_time);

        label.as_ref().set_widget_name(Self::NAME);
        label.as_ref().add_css_class("module");

        Self { label }
    }

    fn formatted_time(format: &str) -> String {
        let time = Local::now();
        time.format(format).to_string()
    }

    async fn background_task(config: &ClockConfig, sender: Sender<String>) {
        let format = &config.format;

        loop {
            let formatted_time = Clock::formatted_time(format);
            let _ = sender.send(formatted_time);

            sleep(Duration::from_secs(10)).await;
        }
    }
}
