use crate::{
    common::Metadata,
    modules::{Battery, ModuleFactory},
};
use chameleon_config::panel::BatteryConfig;
use grapes::{RespawnableTask, state, tokio::sync::broadcast};
use std::sync::{LazyLock, RwLock};

pub static CHARGE_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| broadcast::Sender::new(64));

static CHARGE_TASK: RwLock<RespawnableTask<()>> =
    RwLock::new(RespawnableTask::new());

pub struct BatteryFactory;

impl ModuleFactory for BatteryFactory {
    type Config = BatteryConfig;
    type Module = Battery;

    fn create(config: &Self::Config, _meta: &Metadata) -> Self::Module {
        CHARGE_TASK.write().unwrap().spawn();

        let formatted_charge = state("".to_string());
        formatted_charge.track(&CHARGE_SENDER);

        Battery::new(&formatted_charge)
    }
}
