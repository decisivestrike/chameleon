mod cli;
mod config;
mod dbus;
pub use dbus::*;
pub mod manager;
pub mod queue;
pub mod window;

use crate::cli::Args;
use crate::config::Rules;
use crate::manager::NotificationManager;
use chameleon_shared::utils::read_config;
use chameleon_shared::{CHAMELEON_CONFIG_ROOT, init_tracing_subscriber};
use gtk::glib;
use gtke::Css;
use gtke::css::StylePriority;
pub use server::NotificationServer;
use std::path::PathBuf;
use std::process::exit;
use std::sync::LazyLock;
use tracing::error;

pub static CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CONFIG_ROOT.join("notifications.toml"));

pub static STYLES_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CONFIG_ROOT.join("styles.css"));

fn main() {
    init_tracing_subscriber();

    let args: Args = argh::from_env();

    let config_path = args.config_path.as_ref().unwrap_or(&CONFIG_PATH);
    let config: Rules = read_config(config_path).unwrap();

    let styles_path = args.styles_path.as_ref().unwrap_or(&STYLES_PATH);
    Css::load(styles_path).apply(StylePriority::User);

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let manager = NotificationManager::new(config);
    manager.run();

    glib::MainLoop::new(None, false).run();
}
