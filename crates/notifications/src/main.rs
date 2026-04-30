mod config;
pub mod manager;
pub mod notification;
pub mod queue;
pub mod requests;
pub mod responses;
pub mod server;
pub mod window;

use crate::config::NotificationsConfig;
use crate::manager::NotificationManager;
use chameleon_core::{
    DEFAULT_CONFIG_FOLDER, init_tracing_subscriber, read_config,
};
use gtk::glib;
pub use server::NotificationServer;
use std::process::exit;
use std::sync::LazyLock;
use tracing::error;

pub static CONFIG: LazyLock<NotificationsConfig> = LazyLock::new(|| {
    let config_name = "notifications.toml";
    let config_path = DEFAULT_CONFIG_FOLDER.join(config_name);

    read_config(config_path)
});

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";

fn main() {
    init_tracing_subscriber();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let manager = NotificationManager::new();
    manager.run();

    glib::MainLoop::new(None, false).run();
}
