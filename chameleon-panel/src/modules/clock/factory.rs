use crate::{
    common::Metadata,
    modules::{Clock, ModuleFactory},
};
use chameleon_config::panel::ClockConfig;
use chrono::Local;
use grapes::{
    RespawnableTask, state,
    tokio::sync::{RwLock, broadcast},
};
use std::sync::LazyLock;

pub static FTIME_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| broadcast::Sender::new(64));

static FTIME_TASK: RwLock<RespawnableTask<()>> =
    RwLock::const_new(RespawnableTask::new());

pub struct ClockFactory;

impl ModuleFactory for ClockFactory {
    type Config = ClockConfig;
    type Module = Clock;

    fn create(config: &Self::Config, _meta: &Metadata) -> Self::Module {
        FTIME_TASK.blocking_write().spawn();

        let formatted_time = state(Local::now().to_string());
        formatted_time.track(&FTIME_SENDER);

        Clock::new(&formatted_time)
    }
}
