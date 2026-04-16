mod config;
pub mod manager;
pub mod notification;
pub mod queue;
pub mod requests;
pub mod responses;
pub mod server;
pub mod window;

use chameleon_core::{DEFAULT_CONFIG_FOLDER, read_config};
pub use server::NotificationServer;

use crate::config::NotificationsConfig;
use std::sync::LazyLock;
use std::time::Duration;

pub static CONFIG: LazyLock<NotificationsConfig> = LazyLock::new(|| {
    let config_name = "seagull.toml";
    let config_path = DEFAULT_CONFIG_FOLDER.join(config_name);

    read_config(config_path)
});

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";
pub const ICON_SIZE: i32 = 64;

pub static DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);
pub static MAX_NOTIFICATIONS: usize = 5;
pub static GAP: i32 = 10;
