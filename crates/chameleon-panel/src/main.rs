mod cli;
pub mod config;
pub mod modules;
pub mod panel;
pub mod services;

use crate::cli::Args;
use crate::config::Rules;
use crate::panel::Panel;
use chameleon_shared::css::{Css, StylePriority};
use chameleon_shared::init_tracing_subscriber;
use chameleon_shared::utils::read_config;
use chameleon_shared::watcher::FilesWatcher;
use gtk::gdk::prelude::DisplayExt;
use gtk::gdk::{self};
use gtk::glib;
use gtk::glib::object::Cast;
use gtk::prelude::GtkWindowExt;
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

    let display = gdk::Display::default().expect("No display");

    for monitor in display
        .monitors()
        .into_iter()
        .filter_map(|obj| obj.ok()?.downcast::<gdk::Monitor>().ok())
    {
        let panel = Panel::new(rules.clone(), monitor);
        panel.present();

        // let connector_name = monitor.connector().unwrap().to_string();
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
