use chameleon_config::{self as config};
use chrono::{DateTime, Local};
use config::bar::modules::Clock as ClockConfig;
use grapes::{
    Component, Reactive, derived,
    gtk::{self, Label, prelude::WidgetExt},
    subscriber, task,
    tokio::time::sleep,
};
use std::{rc::Rc, time::Duration};

#[derive(Clone, Debug, Component)]
pub struct Clock {
    #[root]
    label: gtk::Label,
}

impl Clock {
    const NAME: &str = "clock";

    pub fn new(config: Rc<ClockConfig>) -> Self {
        let time = subscriber(&task(async |sender| {
            let duration = Duration::from_secs(1);

            loop {
                let time = Local::now();

                if let Err(e) = sender.send(time) {
                    log::error!("{e}");
                };

                sleep(duration).await;
            }
        }));

        let formatter_time =
            derived(move || Clock::format(*time.get(), &config.format));

        let label = Label::statefull(&formatter_time);
        label.set_widget_name(Self::NAME);
        label.add_css_class("module");

        Self { label }
    }

    fn format(time: DateTime<Local>, format: &str) -> String {
        time.format(format).to_string()
    }
}
