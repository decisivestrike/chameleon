use chrono::{DateTime, Local};
use grapes::{
    Component, GtkCompatible,
    gtk::{self, prelude::WidgetExt},
    service,
    tokio::time::sleep,
};
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, GtkCompatible)]
pub struct Clock {
    #[root]
    label: gtk::Label,
    format: &'static String,
}

impl Component for Clock {
    type Message = DateTime<Local>;
    type Props = &'static ClockConfig;

    fn new(props: Self::Props) -> Self {
        let ClockConfig { format } = props;
        let label = gtk::Label::new(None);
        label.add_css_class("module");

        let clock = Self { format, label };

        clock.connect_service::<TimeService>();
        clock
    }

    fn update(&self, time: DateTime<Local>) {
        let time_label = time.format(self.format).to_string();

        self.label.set_label(&time_label);
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ClockConfig {
    #[serde(default = "ClockConfig::default_format")]
    pub format: String,
}

impl ClockConfig {
    fn default_format() -> String {
        "%H:%M".to_string()
    }
}
