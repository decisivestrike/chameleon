mod cli;
pub mod config;
pub mod layer;

use crate::cli::Args;
use crate::config::Configuration;
use crate::layer::WidgetsLayer;
use chameleon_shared::init_tracing_subscriber;
use gtk::gdk::Monitor;
use gtk::glib;
use gtke::css::StylePriority;
use gtke::monitor::GtkeMonitorExt;
use gtke::{Css, WindowComponent};
use std::process::exit;
use tracing::{error, info};

fn main() {
    init_tracing_subscriber();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let Args { styles_path, .. } = argh::from_env();

    info!("Setup widgets...");

    Css::load(&styles_path).apply(StylePriority::User);

    for monitor in Monitor::all().iter() {
        let widgets_layer =
            WidgetsLayer::new(monitor, &Configuration::default());

        widgets_layer.append(
            gtk::Label::builder().label("WOW").name("wow").build(),
            100.0,
            100.0,
        );

        widgets_layer.present();

        // let connector_name = monitor.connector().unwrap().to_string();
        // self.widgets_layers.insert(connector_name, widgets_layer);
    }

    glib::MainLoop::new(None, false).run();
}
