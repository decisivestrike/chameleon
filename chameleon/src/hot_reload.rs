use chameleon_config::Config;
use futures_util::StreamExt;
use grapes::{
    Css, RT,
    css::StylePriority,
    glib::{self, clone},
    gtk,
    tokio::sync::mpsc::{self, Sender},
};
use inotify::{Inotify, WatchMask};
use log::info;
use std::{path::PathBuf, rc::Rc, sync::Arc};

use crate::instance_manager::INSTANCE_MANAGER;

/// Смотрит за конфигами и загружает их при изменениях
pub struct Watcher {
    app: gtk::Application,
    config_path: PathBuf,
    styles_path: PathBuf,
}

impl Watcher {
    pub fn new(
        app: &gtk::Application,
        config_path: PathBuf,
        styles_path: PathBuf,
    ) -> Self {
        Self {
            app: app.clone(),
            config_path,
            styles_path,
        }
    }

    pub fn run(self) {
        let (sender, mut receiver) = mpsc::channel::<()>(16);

        let Self {
            app,
            config_path,
            styles_path,
        } = self;

        RT.spawn(Self::watcher(sender, config_path, styles_path));

        glib::spawn_future_local(async move {
            loop {
                if let Some(_) = receiver.recv().await {
                    match Config::read() {
                        Ok(config) => {
                            let config = Rc::new(config);
                            INSTANCE_MANAGER.configure_modules(&app, &config);
                        }
                        Err(e) => {
                            log::error!("{e}");
                        }
                    }
                }
            }
        });
    }

    /// Spawn local task
    fn on_styles_change(styles_path: &Arc<PathBuf>) {
        glib::idle_add(clone!(
            #[strong]
            styles_path,
            move || {
                Css::load(&*styles_path).apply(StylePriority::User);

                glib::ControlFlow::Break
            }
        ));
        info!("Styles reloaded");
    }

    /// Just sends empty message
    async fn on_config_change(sender: &Sender<()>) {
        sender.send(()).await.unwrap();
        info!("Config reloaded");
    }

    async fn watcher(
        sender: Sender<()>,
        config_path: PathBuf,
        styles_path: PathBuf,
    ) {
        let inotify =
            Inotify::init().expect("Error while initializing inotify instance");

        let config_wd = inotify
            .watches()
            .add(&config_path, WatchMask::CLOSE_WRITE)
            .expect("Failed to add config file watch");

        let styles_wd = inotify
            .watches()
            .add(&styles_path, WatchMask::CLOSE_WRITE)
            .expect("Failed to add styles file watch");

        let mut buffer = [0; 1024];
        let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

        let styles_path = Arc::new(styles_path);

        loop {
            if let Some(maybe_event) = stream.next().await
                && let Ok(event) = maybe_event
            {
                let wd = event.wd;

                if wd == config_wd {
                    Self::on_config_change(&sender).await;
                } else if wd == styles_wd {
                    Self::on_styles_change(&styles_path);
                }
            }
        }
    }
}
