use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::exit;
use std::sync::LazyLock;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

static SOCKET_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/tmp/chameleon"));

static SOCKET_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| SOCKET_FOLDER.join("launcher.sock"));

/// ipc
pub async fn wait_toggle_command(sender: mpsc::Sender<()>) {
    if !SOCKET_FOLDER.exists() {
        fs::create_dir(&*SOCKET_FOLDER).await.unwrap();
    }

    if SOCKET_PATH.exists() {
        fs::remove_file(&*SOCKET_PATH).await.unwrap();
    }

    let listener = UnixListener::bind(&*SOCKET_PATH).unwrap();
    info!("Listening {:?}", *SOCKET_PATH);

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("Connection from {addr:?}");

        let mut lines = BufReader::new(stream).lines();

        match lines.next_line().await {
            Ok(Some(_line)) => {
                sender.send(()).await.unwrap();
            }
            _ => (),
        };
    }
}

/// Sends command to socket
pub fn send_toggle_command() -> ! {
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
