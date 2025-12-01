use chameleon_configuration as config;
use std::time::Duration;

use grapes::{
    Component, GtkCompatible, gtk, service,
    tokio::{self, time::sleep},
};
use log::warn;

const BAT: &str = "BAT1";

#[derive(Clone, Debug, GtkCompatible)]
pub struct Battery {
    #[root]
    label: gtk::Label,
}

impl Battery {
    async fn charge() -> Option<String> {
        let battery_path = format!("/sys/class/power_supply/{}/capacity", BAT);

        match tokio::fs::read_to_string(battery_path).await {
            Ok(capacity) => Some(capacity.trim().to_string()),
            Err(e) => {
                warn!("{e}");
                None
            }
        }
    }
}

impl Component for Battery {
    const NAME: &str = "battery";

    type Message = String;
    type Props = &'static config::bar::Battery;

    fn new(config: Self::Props) -> Self {
        let label = gtk::Label::new(None);

        let battery = Self { label };

        battery.connect_service::<BatteryService>();

        battery
    }

    fn update(&self, charge: String) {
        let label = &(charge + "%");

        self.label.set_label(label);
    }
}

service!(BatteryService -> String, async |tx| {
    loop {
        let charge = Battery::charge().await;

        if let Some(charge) = charge {
            tx.send(charge).unwrap();
        }

        sleep(Duration::from_secs(60)).await;
    }
});
