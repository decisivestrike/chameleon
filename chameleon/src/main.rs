mod bar;
mod core;
mod widgets;

use crate::{bar::Bar, core::cli::Args, widgets::WidgetLayer};
use chameleon_configuration::CONFIG;
use clap::Parser;
use grapes::{
    WindowComponent,
    glib::{self, ExitCode},
    gtk::{
        self,
        gdk::prelude::MonitorExt,
        gio::{
            ApplicationFlags,
            prelude::{ApplicationExt, ApplicationExtManual},
        },
    },
};
use log::info;

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn on_activate(application: &gtk::Application) {
    if CONFIG.bar.enabled {
        info!("Setup bar...");

        for monitor in core::monitors().iter() {
            let bar = Bar::new(application, monitor, &CONFIG.bar);

            bar.present();
        }
    }

    if CONFIG.widgets.enabled {
        info!("Setup widgets...");

        for monitor in core::monitors().iter() {
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

fn load_css(style_path: &str) {
    if CONFIG.widgets.enabled {
        let provider = gtk::CssProvider::new();
        provider.load_from_path("widget-layer.css");

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default()
                .expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_path(style_path);

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() -> glib::ExitCode {
    let Args {
        config_path,
        style_path,
    } = Args::parse();

    init_logger();

    let app = gtk::Application::builder()
        .application_id("decisivestrike.chameleon")
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_command_line(|app, _| {
        app.activate();
        ExitCode::SUCCESS
    });

    app.connect_startup(move |_| load_css(&style_path));
    app.connect_activate(on_activate);

    app.run()
}
