use chameleon_config as config;
use chrono::{DateTime, Local};
use config::bar::modules::Clock as ClockConfig;
use grapes::{
    Component, Connectable, GtkCompatible,
    gtk::{self, prelude::WidgetExt},
    service,
    tokio::time::sleep,
    updateable::Updateable,
};
use std::time::Duration;

#[derive(Clone, Debug, GtkCompatible)]
pub struct Clock {
    #[root]
    label: gtk::Label,
    format: &'static String,
}

impl Updateable for Clock {
    type Message = DateTime<Local>;

    fn update(&self, time: DateTime<Local>) {
        let time_label = time.format(self.format).to_string();
        self.label.set_label(&time_label);
    }
}

impl Component for Clock {
    const NAME: &str = "clock";

    type Props = &'static ClockConfig;

    fn new(props: Self::Props) -> Self {
        let ClockConfig { format } = props;
        let label = gtk::Label::new(None);
        label.add_css_class("module");

        let clock = Self { format, label };

        clock.connect_service::<TimeService>();
        clock
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
