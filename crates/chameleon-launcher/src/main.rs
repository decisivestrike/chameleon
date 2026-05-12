mod card;
mod cli;
mod config;
mod entry_object;
pub mod ipc;
mod launcher;

use crate::cli::Args;
use crate::config::LauncherConfig;
use crate::ipc::{send_toggle_command, wait_toggle_command};
use crate::launcher::Launcher;
use chameleon_shared::utils::read_config;
use chameleon_shared::{init_tracing_subscriber, styles_watcher};
use gtk::glib;
use gtke::Css;
use gtke::css::StylePriority;
use gtkio::future::spawn;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, LazyLock, RwLock};
use tracing::error;

static LAUNCHER: LazyLock<RwLock<Option<Arc<Launcher>>>> =
    LazyLock::new(|| RwLock::new(None));

fn main() {
    init_tracing_subscriber();

    let Args {
        config_path,
        styles_path,
        toggle,
    } = argh::from_env();

    if toggle {
        send_toggle_command()
    }

    if let Err(e) = gtk::init() {
        error!("Failed to initialize GTK: {e}");
        exit(1);
    };

    let config: LauncherConfig = read_config(&config_path).unwrap();

    *LAUNCHER.write().unwrap() = Some(Launcher::create(config));

    let css_path = PathBuf::from(&styles_path);

    Css::load(&css_path).apply(StylePriority::User);

    spawn(wait_toggle_command());
    spawn(styles_watcher(css_path));

    glib::MainLoop::new(None, false).run();
}
