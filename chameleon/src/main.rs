mod bar;
mod cli;
mod hot_reload;
mod widgets;

use crate::{bar::Bar, cli::Args, widgets::WidgetLayer};
use chameleon_config::Config;
use grapes::{
    Css, RT, WindowComponent,
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
    tokio::sync::mpsc,
};
use log::info;
use std::{cell::RefCell, path::Path, rc::Rc};

thread_local! {
    static BARS: RefCell<Vec<Bar>> = RefCell::new(vec![]);
    static WIDGET_LAYER: RefCell<Option<WidgetLayer>> = RefCell::new(None);
}

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

pub fn apply_config(application: &gtk::Application, config: Rc<Config>) {
    let widgets_config = config.widgets.clone();
    let bar_config = config.bar.clone();

    match bar_config.enabled {
        // If it already exists, then we don't do anything.
        true if BARS.with_borrow(|b| b.len() != 0) => (),
        true => {
            info!("Setup bar...");

            for monitor in Monitor::all().iter() {
                let bar = Bar::new(application, monitor, bar_config.clone());
                bar.present();
                BARS.with(|bars| bars.borrow_mut().push(bar));
            }
        }
        false => BARS.with_borrow_mut(|bars| {
            if bars.len() != 0 {
                bars.retain(|bar| {
                    bar.destroy();
                    false
                })
            }
        }),
    }

    match widgets_config.enabled {
        // If it already exists, then we don't do anything.
        true if WIDGET_LAYER.with_borrow(|wl| wl.is_some()) => (),
        true => {
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

                WIDGET_LAYER.with_borrow_mut(|wl| *wl = Some(widget_layer));

                info!(
                    "Widget layer presented on {}",
                    monitor.connector().unwrap()
                );
            }
        }
        false => {
            WIDGET_LAYER.with_borrow_mut(|layer| {
                if let Some(widget_layer) = layer.as_mut() {
                    widget_layer.destroy();
                    *layer = None;
                }
            });
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
        watch,
    } = argh::from_env();

    // TODO: replace ~ on home

    let config = Rc::new(Config::init(&config_path));

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
        #[strong]
        style_path,
        move |_| load_styles(&style_path, config.clone())
    ));

    app.connect_activate(clone!(
        #[strong]
        config,
        move |app| apply_config(app, config.clone())
    ));

    if watch {
        let (tx, mut rx) = mpsc::channel::<()>(16);
        RT.spawn(hot_reload::watcher(tx, config_path, style_path));

        glib::spawn_future_local(clone!(
            #[strong]
            app,
            async move {
                loop {
                    if let Some(_) = rx.recv().await
                        && let Ok(config) = Config::update()
                    {
                        let config = Rc::new(config);
                        apply_config(&app, config);
                    }
                }
            }
        ));
    }

    app.run()
}
