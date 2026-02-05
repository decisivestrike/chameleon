use chameleon_config::bar::Battery as BatteryConfig;
use grapes::{
    Component, Reactive, derived,
    gtk::{Label, prelude::WidgetExt},
    subscriber, task,
    tokio::{self, time::sleep},
};
use log::warn;
use std::{rc::Rc, time::Duration};

const BAT: &str = "BAT1";

#[derive(Clone, Debug, Component)]
pub struct Battery {
    #[root]
    label: Label,
}

impl Drop for Battery {
    fn drop(&mut self) {
        log::info!("Battery dropped")
    }
}

impl Battery {
    pub fn new(config: Rc<BatteryConfig>) -> Self {
        let maybe_charge = subscriber(&task(async |sender| {
            let duration = Duration::from_secs(60);

            loop {
                let charge = Battery::charge().await;
                sender.send(charge).unwrap();
                sleep(duration).await;
            }
        }));

        let formatted_charge = derived(move || {
            let charge = maybe_charge.get().unwrap_or(0);
            Battery::format(charge, &config.icons)
        });

        let label = Label::statefull(&formatted_charge);
        label.add_css_class("module");
        label.set_widget_name("battery");

        Self { label }
    }

    async fn charge() -> Option<u8> {
        let battery_path = format!("/sys/class/power_supply/{}/capacity", BAT);

        match tokio::fs::read_to_string(battery_path).await {
            Ok(raw_charge) => Some(
                raw_charge
                    .trim()
                    .parse::<u8>()
                    .expect(&format!("Can't parse '{}'", raw_charge)),
            ),
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
