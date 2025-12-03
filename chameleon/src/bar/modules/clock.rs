use chameleon_config as config;
use chrono::{DateTime, Local};
use config::bar::modules::Clock as ClockConfig;
use grapes::{
    Component, Connectable, GtkCompatible, Reactive, derived,
    gtk::{self, Label, prelude::WidgetExt},
    service, state,
    tokio::time::sleep,
};
use std::time::Duration;

#[derive(Clone, Debug, GtkCompatible)]
pub struct Clock {
    #[root]
    label: gtk::Label,
}

impl Clock {
    fn format(time: DateTime<Local>, format: &str) -> String {
        time.format(format).to_string()
    }
}

impl Component for Clock {
    const NAME: &str = "clock";
    type Props = &'static ClockConfig;

    fn new(config: Self::Props) -> Self {
        let time = state(Local::now());
        time.connect_service::<TimeService>();

        let formatter_time =
            derived(move || Clock::format(*time.get(), &config.format));

        let label = Label::statefull(&formatter_time);
        label.add_css_class("module");

        Self { label }
    }
}

service!(TimeService -> DateTime<Local>, async |tx| {
    let duration = Duration::from_secs(1);

    loop {
        let time = Local::now();
        tx.send(time).unwrap();
        sleep(duration).await;
    }
});
