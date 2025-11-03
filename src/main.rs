mod bar;
mod core;
mod widgets;

use crate::{bar::Bar, core::config::CONFIG, widgets::WidgetLayer};
use grapes::{
    WindowComponent,
    gtk::{
        self,
        gdk::prelude::MonitorExt,
        gio::prelude::{ApplicationExt, ApplicationExtManual},
    },
};
use gtk::glib::{self};
use log::info;

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn build_ui(application: &gtk::Application) {
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

fn main() -> glib::ExitCode {
    init_logger();

    let app = gtk::Application::builder()
        .application_id("decisivestrike.chameleon")
        .build();

    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_path("widget-layer.css");

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        let provider = gtk::CssProvider::new();
        provider.load_from_path("style.css");

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(|app| build_ui(app));

    app.run()
}
