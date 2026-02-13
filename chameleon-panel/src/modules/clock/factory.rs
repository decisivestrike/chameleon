use crate::{
    common::Metadata,
    modules::{Clock, ModuleFactory},
};
use chameleon_config::panel::ClockConfig;
use chrono::Local;
use grapes::{SubscribableTask, Task, state, tokio::sync::RwLock};
use std::rc::Rc;

pub static FTIME_TASK: RwLock<Option<SubscribableTask<String>>> =
    RwLock::const_new(None);

pub struct ClockFactory;

impl ModuleFactory for ClockFactory {
    type Config = ClockConfig;
    type Module = Clock;

    fn create(config: &Rc<Self::Config>, _meta: &Metadata) -> Self::Module {
        if FTIME_TASK.blocking_read().is_none() {
            let mut ftime_task = FTIME_TASK.blocking_write();
            let time_format = config.format.clone();

            *ftime_task =
                Some(Task::subscribable(async move |sender, token| {
                    loop {
                        let time = Local::now();
                        let formatted_time = Clock::format(time, &time_format);

                        sender.send(formatted_time);

                        if token.is_cancelled() {
                            break;
                        }
                    }
                }));
        }

        let guard = FTIME_TASK.blocking_read();
        let ftime_task = guard.as_ref().expect("already checked");

        let formatted_time = state(Local::now().to_string());
        formatted_time.track(&ftime_task);

        Clock::new(&formatted_time)
    }
}
