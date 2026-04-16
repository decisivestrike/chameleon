use serde::Deserialize;
use std::time::Duration;
use zbus::zvariant::Type;

const DEFAULT_TIMEOUT_MS: i32 = 5000;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "i")]
pub struct Timeout(i32);

impl Default for Timeout {
    fn default() -> Self {
        Self(DEFAULT_TIMEOUT_MS)
    }
}

impl Into<Duration> for Timeout {
    fn into(self) -> Duration {
        match self.0.try_into() {
            Ok(0) => Duration::MAX,
            Ok(ms) => Duration::from_millis(ms),
            _ => Self::default().into(),
        }
    }
}
