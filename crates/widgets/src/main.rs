use crate::config::Configuration;
use crate::layer::WidgetsLayer;
use gtk::gdk::Monitor;
use gtk::glib;
use gtke::WindowComponent;
use gtke::monitor::GtkeMonitorExt;
use std::process::exit;
use tracing::{error, info};

pub mod config;
pub mod layer;

fn main() {
    tracing_subscriber::fmt().without_time().init();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    info!("Setup widgets...");

    for monitor in Monitor::all().iter() {
        let widgets_layer =
            WidgetsLayer::new(monitor, &Configuration::default());

        widgets_layer.append(gtk::Label::new(Some("WOW")), 100.0, 100.0);

        widgets_layer.present();

        // let connector_name = monitor.connector().unwrap().to_string();
        // self.widgets_layers.insert(connector_name, widgets_layer);
    }

    glib::MainLoop::new(None, false).run();
}
