pub mod errors;
pub mod utils;
pub mod watcher;

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

/// User's home directory
pub static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| home_dir().expect("Can't get home directory"));

/// Config root directory
pub static CHAMELEON_CONFIG_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".config/chameleon"));

/// Themes
pub static CHAMELEON_THEMES_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CONFIG_ROOT.join("themes"));

/// Presets
pub static CHAMELEON_PRESETS_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| CHAMELEON_CONFIG_ROOT.join("presets"));

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
