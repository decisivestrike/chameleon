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
use std::{rc::Rc, sync::LazyLock};
use tokio_util::sync::CancellationToken;

pub static FTIME_SENDER: LazyLock<broadcast::Sender<String>> =
    LazyLock::new(|| broadcast::Sender::new(64));

static FTIME_TASK: RwLock<RespawnableTask<()>> =
    RwLock::const_new(RespawnableTask::new());

pub struct ClockFactory;

impl ModuleFactory for ClockFactory {
    type Config = ClockConfig;
    type Module = Clock;

    fn create(config: &Rc<Self::Config>, _meta: &Metadata) -> Self::Module {
        if !FTIME_TASK.blocking_read().is_ready() {
            let time_format = config.format.clone();

            FTIME_TASK.blocking_write().set(
                async move |token: CancellationToken| {
                    loop {
                        let time = Local::now();
                        let formatted_time =
                            time.format(&time_format).to_string();

                        // If dont have subs
                        if let Err(_) = FTIME_SENDER.send(formatted_time) {
                            break;
                        }

                        if token.is_cancelled() {
                            break;
                        }
                    }
                },
            );
        }

        FTIME_TASK.blocking_write().spawn();

        let formatted_time = state(Local::now().to_string());
        formatted_time.track(&FTIME_SENDER);

        Clock::new(&formatted_time)
    }
}
