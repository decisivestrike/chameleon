use crate::config::Module;
use crate::state::State;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::exit;
use std::sync::LazyLock;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

static SOCKET_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/tmp/chameleon"));

static SOCKET_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| SOCKET_FOLDER.join("main.sock"));

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Health,
    #[allow(dead_code)]
    Restart(Module),
    RestartAll,
}

/// ipc
pub async fn wait_command() {
    if !SOCKET_FOLDER.exists() {
        fs::create_dir(&*SOCKET_FOLDER).await.unwrap();
    }

    if SOCKET_PATH.exists() {
        fs::remove_file(&*SOCKET_PATH).await.unwrap();
    }

    let listener = UnixListener::bind(&*SOCKET_PATH).unwrap();
    info!("Listening {:?}", *SOCKET_PATH);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let mut lines = BufReader::new(stream).lines();

        match lines.next_line().await {
            Ok(Some(line)) => {
                let command = match line.as_str() {
                    "health" => Some(Command::Health),
                    "restartall" => Some(Command::RestartAll),
                    cmd => {
                        warn!("Unknown command: {}", cmd);
                        None
                    }
                };

                if let Some(command) = command {
                    match command {
                        Command::RestartAll => {
                            State::load_config_and_recreate().await
                        }
                        Command::Restart(module) => {
                            State::restart_module(module).await;
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        };
    }
}

pub fn send_command(command: Command) {
    let mut stream = match UnixStream::connect(&*SOCKET_PATH) {
        Ok(stream) => stream,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    let command = match command {
        Command::Health => "health".to_string(),
        Command::Restart(module) => format!("restart {}", module.as_ref()),
        Command::RestartAll => "restartall".to_string(),
    };

    if let Err(e) = stream.write_all(&command.as_bytes()) {
        error!("{e}");
    }
}

/// Sends command to socket
pub fn send_command_with_exit(command: Command) -> ! {
    let mut stream = match UnixStream::connect(&*SOCKET_PATH) {
        Ok(stream) => stream,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };

    let command = match command {
        Command::Health => "health".to_string(),
        Command::Restart(module) => format!("restart {}", module.as_ref()),
        Command::RestartAll => "restartall".to_string(),
    };

    if let Err(e) = stream.write_all(&command.as_bytes()) {
        error!("{e}");
        exit(1);
    }

    exit(0);
}
