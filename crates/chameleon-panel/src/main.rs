use crate::cli::Args;
use crate::config::CONFIG;
use crate::panel::Panel;
use chameleon_shared::init_tracing_subscriber;
use chameleon_shared::utils::read_config;
use chameleon_shared::watcher::FilesWatcher;
use dashmap::DashMap;
use gtk::gdk::Monitor;
use gtk::gdk::prelude::MonitorExt;
use gtk::glib;
use gtke::css::StylePriority;
use gtke::monitor::GtkeMonitorExt;
use gtke::{Css, WindowComponent};
use std::process::exit;
use std::sync::LazyLock;
use tracing::{debug, error, info};

mod cli;
pub mod common;
pub mod config;
pub mod modules;
pub mod panel;
pub mod services;

static PANELS: LazyLock<DashMap<String, Panel>> = LazyLock::new(DashMap::new);

fn main() {
    init_tracing_subscriber();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let Args {
        styles_path,
        config_path,
    } = argh::from_env();

    match read_config(&config_path) {
        Ok(config) => {
            debug!("{:#?}", config);
            CONFIG.set(config).unwrap()
        }
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    Css::load(&styles_path).apply(StylePriority::User);

    info!("Setup panels...");

    for monitor in Monitor::all().iter() {
        let panel = Panel::new(monitor, config::config());
        panel.present();

        let connector_name = monitor.connector().unwrap().to_string();
        PANELS.insert(connector_name, panel);
    }

    FilesWatcher::new()
        .unwrap()
        .add_stylesheet(styles_path)
        .unwrap()
        .run();

    glib::MainLoop::new(None, false).run();
}
