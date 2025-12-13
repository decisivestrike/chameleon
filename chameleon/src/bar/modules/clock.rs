use chameleon_config::{self as config, bar};
use chrono::{DateTime, Local};
use config::bar::modules::Clock as ClockConfig;
use grapes::{
    Component, Connectable, GtkCompatible, Reactive, broadcast, derived,
    gtk::{self, Label, prelude::WidgetExt},
    state,
    tokio::time::sleep,
};
use std::{rc::Rc, time::Duration};

use crate::bar::AsBarModule;

#[derive(Clone, Debug, GtkCompatible)]
pub struct Clock {
    #[root]
    label: gtk::Label,
}

impl Clock {
    pub fn new(config: Rc<ClockConfig>) -> Self {
        let time = state(Local::now());
        time.connect_service::<TimeService>();

        let formatter_time =
            derived(move || Clock::format(*time.get(), &config.format));

        let label = Label::statefull(&formatter_time);
        label.add_css_class("module");

        Self { label }
    }

    fn format(time: DateTime<Local>, format: &str) -> String {
        time.format(format).to_string()
    }
}

impl Component for Clock {
    const NAME: &str = "clock";
}

impl AsBarModule for Clock {
    fn as_module(&self) -> bar::Module {
        bar::Module::Clock
    }
}

broadcast!(TimeService -> DateTime<Local>, async |tx| {
    let duration = Duration::from_secs(1);

    loop {
        let time = Local::now();
        tx.send(time).unwrap();
        sleep(duration).await;
    }
});
