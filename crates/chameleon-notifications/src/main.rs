mod cli;
mod config;

mod dbus;
pub use dbus::*;

pub mod history;
pub mod manager;
pub mod window;
mod windows_map;
pub use server::NotificationServer;

use crate::cli::Args;
use crate::config::Rules;
use crate::manager::NotificationManager;
use chameleon_shared::init_tracing_subscriber;
use chameleon_shared::utils::read_config;
use chameleon_shared::watcher::FilesWatcher;
use gtk::glib::{self};
use gtke::Css;
use gtke::css::StylePriority;
use std::process::exit;
use tracing::error;

fn main() {
    init_tracing_subscriber();

    if let Err(e) = gtk::init() {
        error!("Failed to initialize GTK: {e}");
        exit(1);
    };

    let Args {
        config_path,
        styles_path,
    } = argh::from_env();

    let config: Rules = match read_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    Css::load(&styles_path).apply(StylePriority::User);

    FilesWatcher::new()
        .unwrap()
        .add_stylesheet(styles_path)
        .unwrap()
        .run();

    let manager = NotificationManager::new(config);
    manager.run();

    glib::MainLoop::new(None, false).run();
}
