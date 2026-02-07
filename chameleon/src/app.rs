use crate::{cli::Args, hot_reload::Watcher};
use chameleon_config::Config;
use chameleon_panel::Panel;
use chameleon_widgets::WidgetLayer;
use grapes::{
    Css, WindowComponent,
    css::StylePriority,
    gio::ApplicationFlags,
    glib::{ExitCode, clone},
    gtk::{self, gdk::Monitor},
    prelude::{
        ApplicationExt, ApplicationExtManual, MonitorExt,
        monitor::GrapesMonitorExt,
    },
};
use log::info;
use std::{cell::RefCell, path::Path, rc::Rc};

thread_local! {
    static BARS: RefCell<Vec<Panel>> = RefCell::new(vec![]);
    static WIDGET_LAYER: RefCell<Option<WidgetLayer>> = RefCell::new(None);
}

pub struct Chameleon {
    app: gtk::Application,
}

impl Chameleon {
    pub fn new(args: Args) -> Self {
        let Args {
            config_path,
            style_path,
            watch,
        } = args;

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
            move |_| Self::load_styles(&style_path, config.as_ref())
        ));

        app.connect_activate(clone!(
            #[strong]
            config,
            move |app| Self::apply_config(app, config.clone())
        ));

        if watch {
            let watcher = Watcher::new(&app, config_path, style_path);
            watcher.run();
        }

        Self { app }
    }

    pub fn run(&self) -> ExitCode {
        self.app.run()
    }

    pub fn apply_config(application: &gtk::Application, config: Rc<Config>) {
        let widgets_config = config.widgets.clone();
        let bar_config = config.panel.clone();

        match bar_config.enabled {
            true if BARS.with_borrow(|b| b.len() != 0) => {
                BARS.with_borrow_mut(|bars| {
                    bars.iter_mut()
                        .for_each(|bar| bar.apply_config(config.panel.clone()))
                });
            }
            true => {
                info!("Setup bar...");

                for monitor in Monitor::all().iter() {
                    let bar =
                        Panel::new(application, monitor, bar_config.clone());
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

    fn load_styles(style_path: impl AsRef<Path>, config: &Config) {
        Css::load(style_path).apply(StylePriority::User);

        if config.widgets.enabled {
            Css::from_str(include_str!("../../styles/widget-layer.css"))
                .apply(StylePriority::User);
        }
    }
}
