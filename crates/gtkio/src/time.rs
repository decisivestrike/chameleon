use glib::SourceId;
use std::time::Duration;

pub fn seconds(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

pub fn timeout_local(delay: Duration, f: impl FnOnce() + 'static) -> SourceId {
    if delay.subsec_nanos() == 0 {
        let secs = delay.as_secs() as u32;
        glib::timeout_add_seconds_local_once(secs, f)
    } else {
        glib::timeout_add_local_once(delay, f)
    }
}

pub fn interval_local(
    delay: Duration,
    f: impl FnMut() -> glib::ControlFlow + 'static,
) -> SourceId {
    if delay.subsec_nanos() == 0 {
        let secs = delay.as_secs() as u32;
        glib::timeout_add_seconds_local(secs, f)
    } else {
        glib::timeout_add_local(delay, f)
    }
}
