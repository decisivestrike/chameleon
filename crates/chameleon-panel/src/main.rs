use crate::config::Configuration;
use crate::panel::Panel;
use chameleon_core::init_tracing_subscriber;
use dashmap::DashMap;
use grapes::WindowComponent;
use grapes::prelude::MonitorExt;
use gtk::gdk::Monitor;
use gtk::glib;
use gtke::monitor::GtkeMonitorExt;
use std::process::exit;
use std::sync::LazyLock;
use tracing::{error, info};

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

    info!("Setup panels...");

    for monitor in Monitor::all().iter() {
        let panel = Panel::new(monitor, &Configuration::default());
        panel.present();

        let connector_name = monitor.connector().unwrap().to_string();
        PANELS.insert(connector_name, panel);
    }

    glib::MainLoop::new(None, false).run();
}
