mod cli;
mod config;
mod dbus;
use arc_swap::ArcSwap;
use chameleon_shared::watcher::{FilesWatcher, watcher};
pub use dbus::*;
pub mod history;
pub mod manager;
pub mod queue;
pub mod window;
mod windows_map;
pub use server::NotificationServer;

use crate::cli::Args;
use crate::config::Rules;
use crate::manager::NotificationManager;
use chameleon_shared::init_tracing_subscriber;
use chameleon_shared::utils::read_config;
use gtk::glib::{self, clone};
use gtke::Css;
use gtke::css::StylePriority;
use gtkio::future::spawn;
use std::collections::HashMap;
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

    let config: ArcSwap<Rules> = match read_config(&config_path) {
        Ok(config) => ArcSwap::from_pointee(config),
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    Css::load(&styles_path).apply(StylePriority::User);

    FilesWatcher::new()
        .unwrap()
        .add_stylesheet(styles_path)
        .add(
            config_path,
            Box::new(clone!(
                #[strong]
                config,
                move |path| {
                    let new_config: Rules = read_config(path).unwrap();
                }
            )),
        )
        .run();

    let manager = NotificationManager::new(config);
    manager.run();

    glib::MainLoop::new(None, false).run();
}
