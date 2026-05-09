use crate::common::Metadata;
use crate::config::{CONFIG, ClockConfig};
use crate::modules::{BaseModule, ModuleFactory};
use chrono::Local;
use gtk::prelude::WidgetExt;
use gtke::Component;
use gtkio::future::spawn;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;

pub static TIME_SENDER: LazyLock<watch::Sender<String>> = LazyLock::new(|| {
    let initial_time = Clock::formatted_time(&CONFIG.clock.format);
    let sender = watch::Sender::new(initial_time);

    spawn(Clock::background_task(&CONFIG.clock, sender.clone()));

    sender
});

#[derive(Debug, Component)]
pub struct Clock {
    #[root]
    base: BaseModule,
}

impl ModuleFactory for Clock {
    type Config = ClockConfig;

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        let clock = Clock::new();

        Ok(Rc::new(clock))
    }
}

impl Clock {
    const NAME: &str = "clock";

    pub fn new() -> Self {
        let state = TIME_SENDER.subscribe();
        let base = BaseModule::new(state);

        base.set_widget_name(Self::NAME);
        base.add_css_class("module");

        Self { base }
    }

    fn formatted_time(format: &str) -> String {
        let time = Local::now();
        time.format(format).to_string()
    }

    async fn background_task(
        config: &ClockConfig,
        sender: watch::Sender<String>,
    ) {
        let format = &config.format;

        loop {
            let formatted_time = Clock::formatted_time(format);
            sender.send(formatted_time).unwrap();

            sleep(Duration::from_secs(1)).await;
        }
    }
}
