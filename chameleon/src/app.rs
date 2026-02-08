use crate::{
    cli::Args, hot_reload::Watcher, instance_manager::INSTANCE_MANAGER,
};
use chameleon_config::Config;
use grapes::{
    Css,
    css::StylePriority,
    gio::ApplicationFlags,
    glib::{ExitCode, clone},
    gtk::{self},
    prelude::{ApplicationExt, ApplicationExtManual},
};
use std::rc::Rc;

const APPLICATION_ID: &str = "decisivestrike.chameleon";

pub struct Chameleon {
    app: gtk::Application,
}

impl Chameleon {
    pub fn new(args: Args) -> Self {
        let Args {
            config_path,
            styles_path: style_path,
            watch,
        } = args;

        let app = gtk::Application::builder()
            .application_id(APPLICATION_ID)
            .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        app.connect_command_line(|app, _| {
            app.activate();
            ExitCode::SUCCESS
        });

        app.connect_startup(clone!(
            #[strong]
            style_path,
            move |_| Css::load(&style_path).apply(StylePriority::User)
        ));

        let config = Rc::new(Config::init(&config_path));
        app.connect_activate(clone!(
            #[strong]
            config,
            move |app| INSTANCE_MANAGER.configure_modules(app, &config)
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
}
