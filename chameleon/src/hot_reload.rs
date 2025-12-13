use futures_util::StreamExt;
use grapes::{
    Css,
    css::StylePriority,
    glib::{self, clone},
};
use inotify::{Inotify, WatchMask};
use log::info;
use std::{path::PathBuf, sync::Arc};

pub async fn watcher(config_path: PathBuf, styles_path: PathBuf) {
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
            match event.wd {
                wd if wd == config_wd => info!("fake config update"),
                wd if wd == styles_wd => {
                    glib::idle_add(clone!(
                        #[strong]
                        styles_path,
                        move || {
                            Css::load(&*styles_path)
                                .apply(StylePriority::Application);

                            glib::ControlFlow::Break
                        }
                    ));

                    info!("Styles reloaded");
                }
                _ => (),
            }
        }
    }
}
