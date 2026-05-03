pub mod config;
pub mod errors;
pub mod utils;

use futures::StreamExt;
use gtk::glib::{self, clone};
use gtke::Css;
use gtke::css::StylePriority;
use inotify::{Inotify, WatchMask};
use std::env::home_dir;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Config root directory
pub static CHAMELEON_CFG_ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    let maybe_home = home_dir();

    if let Some(home) = maybe_home {
        home.join(".config/chameleon")
    } else {
        panic!("Can't get $HOME directory")
    }
});

/// Themes
pub static CHAMELEON_THEMES_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CFG_ROOT.join("themes"));

/// Presets
pub static CHAMELEON_PRESETS_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CFG_ROOT.join("presets"));

/// Reads toml file

pub async fn styles_watcher(styles_path: PathBuf) {
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
            glib::idle_add_once(clone!(
                #[strong]
                styles_path,
                move || {
                    Css::load(&styles_path).apply(StylePriority::User);
                    info!("Styles updated");
                }
            ));
        }
    }
}

/// Setup subscriber based on environment variable
///
/// Default level: error
pub fn init_tracing_subscriber() {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .init();
}
