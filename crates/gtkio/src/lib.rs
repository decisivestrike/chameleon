pub mod future;
pub mod time;
pub mod workers;

use std::sync::LazyLock;
use tokio::runtime::Runtime;

static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("cant create runtime"));

#[cfg(test)]
mod tests {
    use crate::future::*;

    #[test]
    fn sum_callback() {
        spawn_with_local_callback(async { 2 + 2 }, |sum| assert_eq!(sum, 4));
    }
}
