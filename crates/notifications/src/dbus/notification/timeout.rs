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
        if let Ok(ms) = self.0.try_into() {
            match ms {
                0 => Duration::MAX,
                ms => Duration::from_millis(ms),
            }
        } else {
            Self::default().into()
        }
    }
}
