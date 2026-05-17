use crate::config::BatteryRules;
use crate::modules::{Metadata, PanelModule};
use gtk::glib::clone::Downgrade;
use gtk::glib::object::Cast;
use gtk::glib::{self, ControlFlow, clone};
use std::rc::Rc;

const BAT: &str = "BAT1";

pub struct Battery;

impl PanelModule for Battery {
    type Rules = BatteryRules;

    fn create(
        rules: Self::Rules,
        meta: Rc<Metadata>,
    ) -> Result<gtk::Widget, super::Error> {
        let battery = gtk::Label::builder()
            .name("battery")
            .css_classes(["module"])
            .build();

        glib::timeout_add_seconds_local(60, {
            let battery_weak = battery.downgrade();
            let rules = Rc::new(rules);
            move || {
                if let Some(battery) = battery_weak.upgrade() {
                    glib::spawn_future_local(clone!(
                        #[strong]
                        rules,
                        async move {
                            if let Ok(charge) =
                                Battery::formatted_charge(BAT, &rules.icons)
                                    .await
                            {
                                battery.set_label(&charge);
                            }
                        }
                    ));

                    ControlFlow::Continue
                } else {
                    ControlFlow::Break
                }
            }
        });

        Ok(battery.upcast())
    }
}

impl Battery {
    const NAME: &str = "battery";

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
}
