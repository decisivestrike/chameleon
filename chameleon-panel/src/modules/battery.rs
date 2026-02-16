use crate::{common::Metadata, modules::ModuleFactory};
use anyhow::bail;
use chameleon_config::{CONFIG, panel::BatteryConfig};
use grapes::{
    Component, RT, State,
    gtk::prelude::WidgetExt,
    state,
    tokio::{
        self,
        sync::broadcast::{self},
        time::sleep,
    },
};
use grapes_components::StatefullLabel;
use log::warn;
use std::{rc::Rc, sync::LazyLock, time::Duration};

pub static CHARGE_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| {
        let sender = broadcast::Sender::new(64);

        RT.spawn(Battery::background_task(
            &CONFIG.panel.battery,
            sender.clone(),
        ));

        sender
    });

const BAT_PLACEHOLDER: &str = "BAT1";

#[derive(Debug, Component)]
pub struct Battery {
    #[root]
    label: StatefullLabel<String>,
}

impl ModuleFactory for Battery {
    type Config = BatteryConfig;

    fn create(
        config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        match RT
            .block_on(Self::formatted_charge(BAT_PLACEHOLDER, &config.icons))
        {
            Some(formatted_charge) => {
                let fcs = state(formatted_charge);
                fcs.track(&CHARGE_SENDER);

                Ok(Rc::new(Battery::new(&fcs)))
            }
            None => bail!("I can't find the battery in your device"),
        }
    }
}

impl Battery {
    fn new(formatted_charge: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(&formatted_charge);

        label.as_ref().add_css_class("module");
        label.as_ref().set_widget_name("battery");

        Self { label }
    }

    async fn formatted_charge(
        bat: &str,
        icons: &Vec<String>,
    ) -> Option<String> {
        Battery::charge(bat)
            .await
            .map(|charge| Battery::format(charge, icons))
    }

    async fn charge(bat: &str) -> Option<u8> {
        let battery_path = format!("/sys/class/power_supply/{}/capacity", bat);

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

    async fn background_task(
        config: &'static BatteryConfig,
        sender: broadcast::Sender<String>,
    ) {
        loop {
            if let Some(formatted_charge) =
                Battery::formatted_charge(BAT_PLACEHOLDER, &config.icons).await
            {
                let _ = sender.send(formatted_charge);
            }

            sleep(Duration::from_secs(60)).await;
        }
    }
}
