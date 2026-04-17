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

pub static CONFIG: LazyLock<NotificationsConfig> = LazyLock::new(|| {
    let config_name = "notifications.toml";
    let config_path = DEFAULT_CONFIG_FOLDER.join(config_name);

    read_config(config_path)
});

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";
