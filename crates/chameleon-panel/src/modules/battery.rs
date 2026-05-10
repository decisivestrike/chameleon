use crate::common::Metadata;
use crate::config::{BatteryConfig, CONFIG};
use crate::modules::{BaseModule, ModuleFactory};
use gtk::prelude::WidgetExt;
use gtke::Component;
use gtkio::future::spawn;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;
use tokio::{self};

pub static CHARGE_SENDER: LazyLock<watch::Sender<String>> =
    LazyLock::new(|| {
        let sender = watch::Sender::new(format!("0%"));
        spawn(Battery::background_task(&CONFIG.battery, sender.clone()));

        sender
    });

const BAT_PLACEHOLDER: &str = "BAT1";

#[derive(Debug, Component)]
pub struct Battery {
    #[root]
    base: BaseModule,
}

impl ModuleFactory for Battery {
    type Config = BatteryConfig;

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        let battery = Battery::new();

        Ok(Rc::new(battery))
    }
}

impl Battery {
    const NAME: &str = "battery";

    fn new() -> Self {
        let state = CHARGE_SENDER.subscribe();
        let base = BaseModule::new(state);

        base.as_ref().set_widget_name(Self::NAME);
        base.as_ref().add_css_class("module");

        Self { base }
    }

    async fn formatted_charge(
        bat: &str,
        icons: &Vec<String>,
    ) -> anyhow::Result<String> {
        let charge = Battery::charge(bat).await?;

        Ok(Battery::format(charge, icons))
    }

    async fn charge(bat: &str) -> anyhow::Result<u8> {
        let battery_path = format!("/sys/class/power_supply/{}/capacity", bat);

        let charge_str = tokio::fs::read_to_string(battery_path).await?;

        let charge = charge_str
            .trim()
            .parse::<u8>()
            .expect(&format!("Can't parse '{}'", charge_str));

        Ok(charge)
    }

    fn format(charge: u8, icons: &Vec<String>) -> String {
        let divider = 100.0 / icons.len() as f32;
        let i = (charge as f32 / divider).round() as usize;

        format!("{} {}%", icons[i], charge)
    }

    async fn background_task(
        config: &'static BatteryConfig,
        sender: watch::Sender<String>,
    ) {
        loop {
            if let Ok(charge) =
                Battery::formatted_charge(BAT_PLACEHOLDER, &config.icons).await
            {
                let _ = sender.send(charge);
            }

            sleep(Duration::from_secs(60)).await;
        }
    }
}
