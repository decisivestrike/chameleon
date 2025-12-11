mod bar;
mod cli;
mod widgets;

use std::path::Path;

use crate::{bar::Bar, cli::Args, widgets::WidgetLayer};
use chameleon_config::Config;
use grapes::{
    Css, WindowComponent,
    css::StylePriority,
    glib::{self, ExitCode},
    gtk::{
        self,
        gdk::{Monitor, prelude::MonitorExt},
        gio::{
            ApplicationFlags,
            prelude::{ApplicationExt, ApplicationExtManual},
        },
    },
    prelude::GrapesMonitorExt,
};
use log::info;

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn on_activate(application: &gtk::Application) {
    let config = Config::read();

    if config.bar.enabled {
        info!("Setup bar...");

        for monitor in Monitor::all().iter() {
            let bar = Bar::new(application, monitor, &config.bar);

            bar.present();
        }
    }

    if config.widgets.enabled {
        info!("Setup widgets...");

        for monitor in Monitor::all().iter() {
            let widget_layer = WidgetLayer::new(application, monitor);

            {
                let l = gtk::Label::new(Some("Drag Me!"));
                widget_layer.append(&l, 50.0, 50.0);
            }

            {
                let l = gtk::Label::new(Some("Drag Me Too!"));
                widget_layer.append(&l, 100.0, 100.0);
            }

            {
                let l = gtk::Label::new(Some("Pretty good!"));
                widget_layer.append(&l, 150.0, 150.0);
            }

            widget_layer.present();

            info!("Widget layer presented on {}", monitor.connector().unwrap());
        }
    }

    info!("Ready!");
}

fn load_styles(style_path: impl AsRef<Path>) {
    Css::load(style_path).apply(StylePriority::Application);

    if Config::read().widgets.enabled {
        Css::from_str(include_str!("../../styles/widget-layer.css"))
            .apply(StylePriority::User);
    }
}

fn main() -> glib::ExitCode {
    init_logger();

    let Args {
        config_path,
        style_path,
    } = argh::from_env();

    // replace ~ on home

    Config::init(config_path);

    let app = gtk::Application::builder()
        .application_id("decisivestrike.chameleon")
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_command_line(|app, _| {
        app.activate();
        ExitCode::SUCCESS
    });

    app.connect_startup(move |_| load_styles(&style_path));
    app.connect_activate(on_activate);

    app.run()
}
