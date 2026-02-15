mod factory;
pub use factory::BatteryFactory;

use grapes::{
    Component, State,
    gtk::prelude::WidgetExt,
    tokio::{self},
};
use grapes_components::StatefullLabel;
use log::warn;
use std::rc::Rc;

const BAT: &str = "BAT1";

#[derive(Debug, Component)]
pub struct Battery {
    #[root]
    label: StatefullLabel<String>,
}

impl Drop for Battery {
    fn drop(&mut self) {
        log::info!("Battery dropped")
    }
}

impl Battery {
    pub fn new(charge: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(&charge);
        label.as_ref().add_css_class("module");
        label.as_ref().set_widget_name("battery");

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
