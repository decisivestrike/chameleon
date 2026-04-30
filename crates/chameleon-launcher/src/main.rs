mod card;
mod cli;
mod config;
mod entry_object;
mod launcher;

use gtk::glib;
use gtkio::future::spawn;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, LazyLock, RwLock};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info};

use crate::cli::Command;
use crate::config::LauncherConfig;
use crate::launcher::Launcher;

static SOCKET_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/tmp/chameleon"));

static SOCKET_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| SOCKET_FOLDER.join("launcher.sock"));

static LAUNCHER: LazyLock<RwLock<Option<Arc<Launcher>>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn toggle_launcher() {
    if let Some(launcher) = &*LAUNCHER.read().unwrap() {
        launcher.toggle_visibility();
    }
}

async fn listen_socket() {
    if !SOCKET_FOLDER.exists() {
        fs::create_dir(&*SOCKET_FOLDER).await.unwrap();
    }

    if SOCKET_PATH.exists() {
        fs::remove_file(&*SOCKET_PATH).await.unwrap();
    }

    let listener = UnixListener::bind(&*SOCKET_PATH).unwrap();
    info!("Listening '{:?}'", *SOCKET_PATH);

    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        let mut lines = BufReader::new(stream).lines();

        match lines.next_line().await {
            Ok(Some(_line)) => {
                glib::idle_add_once(toggle_launcher);
            }
            _ => (),
        }
    }
}

fn main() {
    tracing_subscriber::fmt().without_time().init();

    let args: Command = argh::from_env();

    if args.toggle {
        let mut stream = match UnixStream::connect(&*SOCKET_PATH) {
            Ok(stream) => stream,
            Err(e) => {
                error!("{e}");
                exit(1);
            }
        };

        if let Err(e) = stream.write_all(&[2]) {
            error!("{e}");
            exit(1);
        }

        exit(0);
    }

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    *LAUNCHER.write().unwrap() =
        Some(Launcher::create(LauncherConfig::default()));

    spawn(listen_socket());

    glib::MainLoop::new(None, false).run();
}
