pub mod manager;
pub mod notification;
pub mod queue;
pub mod requests;
pub mod responses;
pub mod window;

pub mod server;
pub use server::NotificationServer;

use std::time::Duration;

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";
pub const ICON_SIZE: i32 = 64;

pub static DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);
pub static MAX_NOTIFICATIONS: usize = 5;
pub static GAP: i32 = 10;
