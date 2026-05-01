mod card;
mod cli;
mod config;
mod entry_object;
pub mod ipc;
mod launcher;

use chameleon_core::{init_tracing_subscriber, read_config, styles_watcher};
use gtk::glib;
use gtke::Css;
use gtke::css::StylePriority;
use gtkio::future::spawn;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, LazyLock, RwLock};
use tracing::error;

use crate::cli::Command;
use crate::config::LauncherConfig;
use crate::ipc::{listen_socket, send_toggle_command};
use crate::launcher::Launcher;

static LAUNCHER: LazyLock<RwLock<Option<Arc<Launcher>>>> =
    LazyLock::new(|| RwLock::new(None));

fn main() {
    init_tracing_subscriber();

    let args: Command = argh::from_env();

    if args.toggle {
        send_toggle_command()
    }

    if let Err(e) = gtk::init() {
        error!("Failed to initialize GTK: {e}");
        exit(1);
    };

    let config: LauncherConfig =
        read_config("/home/inqlog/.config/chameleon/launcher.toml");

    *LAUNCHER.write().unwrap() = Some(Launcher::create(config));

    let css_path = PathBuf::from("/home/inqlog/.config/chameleon/styles.css");

    Css::load(&css_path).apply(StylePriority::User);

    spawn(listen_socket());
    spawn(styles_watcher(css_path));

    glib::MainLoop::new(None, false).run();
}
