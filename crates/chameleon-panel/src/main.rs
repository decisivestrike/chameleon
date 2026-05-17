mod cli;
pub mod config;
pub mod modules;
pub mod panel;
pub mod services;

use crate::cli::Args;
use crate::config::Rules;
use crate::panel::Panel;
use chameleon_shared::init_tracing_subscriber;
use chameleon_shared::utils::read_config;
use chameleon_shared::watcher::FilesWatcher;
use gtk::gdk::Monitor;
use gtk::gdk::prelude::MonitorExt;
use gtk::glib;
use gtke::css::StylePriority;
use gtke::monitor::GtkeMonitorExt;
use gtke::{Css, WindowComponent};
use std::process::exit;
use tracing::{debug, error, info};

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

    let rules: Rules = match read_config(&config_path) {
        Ok(config) => {
            debug!("{:#?}", config);
            config
        }
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    Css::load(&styles_path).apply(StylePriority::User);

    info!("Setup panels...");
    let mut panels = Vec::new();

    for monitor in Monitor::each().into_iter() {
        let panel = Panel::new(rules.clone(), monitor);
        panel.present();

        let connector_name = monitor.connector().unwrap().to_string();
        // panels.insert(connector_name, panel);
        panels.push(panel);
    }

    FilesWatcher::new()
        .unwrap()
        .add_stylesheet(styles_path)
        .unwrap()
        .run();

    glib::MainLoop::new(None, false).run();
}
