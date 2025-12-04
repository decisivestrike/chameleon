use chameleon_config::{self as config};
use grapes::{
    Component, Connectable, GtkCompatible, Reactive, derived,
    gtk::{Label, prelude::WidgetExt},
    service, state,
    tokio::{self, time::sleep},
};
use log::warn;
use std::time::Duration;

const BAT: &str = "BAT1";

#[derive(Clone, Debug, GtkCompatible)]
pub struct Battery {
    #[root]
    label: Label,
}

impl Battery {
    async fn charge() -> Option<u8> {
        let battery_path = format!("/sys/class/power_supply/{}/capacity", BAT);

        match tokio::fs::read_to_string(battery_path).await {
            Ok(charge) => Some(charge.parse().unwrap()),
            Err(e) => {
                warn!("{e}");
                None
            }
        }
    }

    fn format(charge: u8, icons: &Vec<String>) -> String {
        let divider = 100.0 / icons.len() as f32;
        let i = (charge as f32 / divider).round() as usize;

        format!("{} {}%", icons[i], charge)
    }
}

impl Component for Battery {
    const NAME: &str = "battery";
    type Props = &'static config::bar::Battery;

    fn new(config: Self::Props) -> Self {
        let charge = state(0);
        charge.connect_service::<BatteryService>();

        let formatted_charge =
            derived(move || Battery::format(*charge.get(), &config.icons));

        let label = Label::statefull(&formatted_charge);
        label.add_css_class("module");

        Self { label }
    }
}

service!(BatteryService -> u8, async |tx| {
    loop {
        let charge = Battery::charge().await;

        if let Some(charge) = charge {
            tx.send(charge).unwrap();
        }

        sleep(Duration::from_secs(60)).await;
    }
});
