use crate::common::Metadata;
use crate::modules::ModuleFactory;
use chameleon_config::CONFIG;
use chameleon_config::panel::ClockConfig;
use chrono::Local;
use grapes::gtk::prelude::WidgetExt;
use grapes::gtk::{self};
use grapes::prelude::containers::GrapesBoxExt;
use grapes::tokio::sync::broadcast::{self, Sender};
use grapes::tokio::time::sleep;
use grapes::{Component, RT, State, state};
use grapes_components::StatefullLabel;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;

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

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
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
