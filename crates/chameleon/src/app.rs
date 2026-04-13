use crate::instance_manager::INSTANCE_MANAGER;
use chameleon_cli::Args;
use chameleon_config::CONFIG;
use futures_util::StreamExt;
use grapes::css::StylePriority;
use grapes::gio::ApplicationFlags;
use grapes::glib::{self, ExitCode};
use grapes::gtk::{self};
use grapes::prelude::{ApplicationExt, ApplicationExtManual};
use grapes::{Css, RT};
use inotify::{Inotify, WatchMask};
use std::path::PathBuf;

const APPLICATION_ID: &str = "decisivestrike.chameleon";

pub struct Chameleon {
    app: gtk::Application,
}

impl Chameleon {
    pub fn new(args: &'static Args) -> Self {
        let Args {
            styles_path,
            watch_enabled,
            ..
        } = args;

        let app = gtk::Application::builder()
            .application_id(APPLICATION_ID)
            .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        app.connect_command_line(|app, _| {
            app.activate();
            ExitCode::SUCCESS
        });

        app.connect_startup(move |_| {
            Css::load(styles_path).apply(StylePriority::User)
        });

        app.connect_activate(move |app| {
            INSTANCE_MANAGER.configure_modules(app, &CONFIG)
        });

        if *watch_enabled {
            Self::start_styles_watcher(styles_path);
        }

        Self { app }
    }

    pub fn run(&self) -> ExitCode {
        self.app.run()
    }

    fn start_styles_watcher(styles_path: &'static PathBuf) {
        RT.spawn(Self::styles_watcher(styles_path));
    }

    async fn styles_watcher(styles_path: &'static PathBuf) {
        let inotify =
            Inotify::init().expect("Error while initializing inotify instance");

        let styles_wd = inotify
            .watches()
            .add(&styles_path, WatchMask::CLOSE_WRITE)
            .expect("Failed to add styles file watch");

        let mut buffer = [0; 1024];
        let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

        loop {
            if let Some(maybe_event) = stream.next().await
                && let Ok(event) = maybe_event
                && event.wd == styles_wd
            {
                glib::idle_add(move || {
                    Css::load(&*styles_path).apply(StylePriority::User);

                    glib::ControlFlow::Break
                });

                log::info!("Styles reloaded");
            }
        }
    }
}
