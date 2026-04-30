pub mod errors;

use futures::StreamExt;
use gtk::glib::{self, clone};
use gtke::Css;
use gtke::css::StylePriority;
use inotify::{Inotify, WatchMask};
use serde::de::DeserializeOwned;
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fmt, fs};
use tracing::{Level, enabled, error, info, warn};
use tracing_subscriber::EnvFilter;

pub static DEFAULT_CONFIG_FOLDER: LazyLock<PathBuf> = LazyLock::new(|| {
    let maybe_home = home_dir();

    if let Some(home) = maybe_home {
        home.join(".config").join("chameleon")
    } else {
        panic!("Cant get $HOME")
    }
});

pub fn read_config<Config, P>(config_path: P) -> Config
where
    Config: Default + DeserializeOwned,
    P: AsRef<Path> + fmt::Debug,
{
    match fs::read_to_string(&config_path) {
        Ok(str) => parse_config(&str),
        Err(e) => {
            warn!(
                "Failed to open config file at {config_path:?}. {e}. The default configuration will be used."
            );
            Config::default()
        }
    }
}

fn parse_config<Config: DeserializeOwned>(input: &str) -> Config {
    match toml::from_str(&input) {
        Ok(config) => config,
        Err(e) => {
            error!("{e}");
            std::process::exit(-1);
        }
    }
}

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
