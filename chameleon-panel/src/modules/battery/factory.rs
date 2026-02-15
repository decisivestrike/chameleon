use crate::{
    common::Metadata,
    modules::{Battery, ModuleFactory},
};
use chameleon_config::panel::BatteryConfig;
use grapes::{RespawnableTask, state, tokio::sync::broadcast};
use std::{
    rc::Rc,
    sync::{LazyLock, RwLock},
};

pub static CHARGE_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| broadcast::Sender::new(64));

static CHARGE_TASK: RwLock<RespawnableTask<()>> =
    RwLock::new(RespawnableTask::new());

pub struct BatteryFactory;

impl ModuleFactory for BatteryFactory {
    type Config = BatteryConfig;
    type Module = Battery;

    fn create(config: &Rc<Self::Config>, _meta: &Metadata) -> Self::Module {
        if !CHARGE_TASK.read().unwrap().is_ready() {
            CHARGE_TASK.write().unwrap().set(async move |token| {
                loop {
                    let charge = Battery::charge().await.unwrap();
                    let formatted_charge =
                        Battery::format(charge, &config.icons);

                    // If dont have subs
                    if let Err(_) = CHARGE_SENDER.send(formatted_charge) {
                        break;
                    }

                    if token.is_cancelled() {
                        break;
                    }
                }
            });
        }

        CHARGE_TASK.write().unwrap().spawn();

        let formatted_charge = state("".to_string());
        formatted_charge.track(&CHARGE_SENDER);

        Battery::new(&formatted_charge)
    }
}
