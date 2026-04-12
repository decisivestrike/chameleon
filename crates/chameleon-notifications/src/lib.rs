pub mod manager;
pub mod notification;
pub mod queue;
pub mod requests;
pub mod responses;
pub mod window;

pub mod server;
pub use server::NotificationServer;

use std::time::Duration;

pub static DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);
pub static MAX_NOTIFICATIONS: usize = 5;
pub static GAP: i32 = 10;
