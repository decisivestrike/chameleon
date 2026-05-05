pub mod future;
pub mod time;
pub mod workers;

use glib::MainContext;
use std::sync::{LazyLock, OnceLock};
use tokio::runtime::{self, Runtime};

/// Defines how many threads should use for background tasks.
///
/// NOTE: The default thread count is 1.
pub static RUNTIME_THREADS: OnceLock<usize> = OnceLock::new();

/// Defines the maximum number of background threads to spawn for handling
/// blocking tasks.
///
/// NOTE: The default max is 64.
pub static RUNTIME_BLOCKING_THREADS: OnceLock<usize> = OnceLock::new();

/// Runtime instance
pub static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(*RUNTIME_THREADS.get_or_init(|| 1))
        .max_blocking_threads(*RUNTIME_BLOCKING_THREADS.get_or_init(|| 64))
        .build()
        .unwrap()
});

/// Glib main context
pub static MAIN_CONTEXT: LazyLock<MainContext> =
    LazyLock::new(MainContext::default);

#[cfg(test)]
mod tests {
    use crate::future::*;

    #[test]
    fn sum_callback() {
        spawn_with_local_callback(async { 2 + 2 }, |sum| assert_eq!(sum, 4));
    }
}
