use chameleon_config::bar::Battery as BatteryConfig;
use grapes::{
    Cacheable, Component, Connectable, GtkCompatible, Reactive, derived,
    gtk::{Label, prelude::WidgetExt},
    persistent, state,
    tokio::{self},
};
use log::warn;
use std::{rc::Rc, time::Duration};

const BAT: &str = "BAT1";

#[derive(Clone, Debug, GtkCompatible)]
pub struct Battery {
    #[root]
    label: Label,
}

impl Battery {
    pub fn new(config: Rc<BatteryConfig>) -> Self {
        let charge = state(0);
        charge.connect_service_unmatched::<BatteryService>(|maybe_charge| {
            maybe_charge.unwrap_or_default()
        });

        let formatted_charge =
            derived(move || Battery::format(*charge.get(), &config.icons));

        let label = Label::statefull(&formatted_charge);
        label.add_css_class("module");
        label.set_widget_name(Self::NAME);

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

impl Component for Battery {
    const NAME: &str = "battery";
}

persistent!(BatteryService -> Option<u8>, Battery::charge, Duration::from_secs(60));
