mod cli;
mod config;
mod entry_object;
pub mod ipc;
mod launcher;
mod providers;

use crate::cli::Args;
use crate::ipc::{send_toggle_command, wait_toggle_command};
use crate::launcher::Launcher;
use chameleon_shared::css::{Css, StylePriority};
use chameleon_shared::utils::read_config;
use chameleon_shared::{init_tracing_subscriber, styles_watcher};
use gtk::glib;
use gtkio::RUNTIME;
use gtkio::future::spawn;
use std::path::PathBuf;
use std::process::exit;
use tokio::sync::mpsc;
use tracing::error;

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

    let config = match read_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    };

    let launcher = Launcher::new(config);
    let (sender, mut receiver) = mpsc::channel::<()>(8);

    glib::spawn_future_local(async move {
        loop {
            if let Some(()) = receiver.recv().await {
                launcher.toggle_visibility()
            }
        }
    });

    RUNTIME.spawn(wait_toggle_command(sender));

    let css_path = PathBuf::from(&styles_path);
    Css::load(&css_path).apply(StylePriority::User);
    spawn(styles_watcher(css_path));

    glib::MainLoop::new(None, false).run();
}
