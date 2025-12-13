mod bar;
mod cli;
mod widgets;

use std::{path::Path, rc::Rc};

use crate::{bar::Bar, cli::Args, widgets::WidgetLayer};
use chameleon_config::Config;
use grapes::{
    Css, WindowComponent,
    css::StylePriority,
    glib::{self, ExitCode, clone},
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

fn on_activate(application: &gtk::Application, config: Rc<Config>) {
    let widgets_config = config.widgets.clone();
    let bar_config = config.bar.clone();

    if bar_config.enabled {
        info!("Setup bar...");

        for monitor in Monitor::all().iter() {
            let bar = Bar::new(application, monitor, bar_config.clone());

            bar.present();
        }
    }

    if widgets_config.enabled {
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

fn load_styles(style_path: impl AsRef<Path>, config: Rc<Config>) {
    Css::load(style_path).apply(StylePriority::Application);

    if config.widgets.enabled {
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

    // TODO: replace ~ on home

    let config = Rc::new(Config::init(config_path));

    let app = gtk::Application::builder()
        .application_id("decisivestrike.chameleon")
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_command_line(|app, _| {
        app.activate();
        ExitCode::SUCCESS
    });

    app.connect_startup(clone!(
        #[strong]
        config,
        move |_| load_styles(&style_path, config.clone())
    ));

    app.connect_activate(clone!(
        #[strong]
        config,
        move |app| on_activate(app, config.clone())
    ));

    app.run()
}
