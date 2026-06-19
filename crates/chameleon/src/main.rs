#![forbid(unsafe_code)]
mod cli;
mod config;

use crate::cli::Args;
use crate::config::Config;
use anyhow::Result;
use chameleon_shared::utils::resolve_path;
use chameleon_shared::watcher::FilesWatcher;
use chameleon_shared::{HOME, init_tracing_subscriber};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::sync::LazyLock;
use tokio::process::Command;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task::spawn_local;
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Default path to chameleon binaries
static CHAMELEON_BIN_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".chameleon/bin"));

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    init_tracing_subscriber();
    run(argh::from_env()).await?;

    Ok(())
}

async fn run(args: Args) -> Result<()> {
    match args.cmd {
        None => start_parent_process().await?,
        Some(_cmd) => (),
    }

    Ok(())
}

async fn start_parent_process() -> Result<()> {
    let Config { modules, theme } =
        Config::load(resolve_path("~/.config/chameleon/config.toml").unwrap());

    if modules.is_empty() {
        warn!("No modules in config. Exit");
        exit(0);
    }

    info!("Modules: {:?}", modules);

    let mut ps = HashMap::new();
    let mut fw = FilesWatcher::new().unwrap();

    let theme_path = theme.map(|t| {
        resolve_path(&format!("~/.config/chameleon/themes/{t}.css"))
            .unwrap()
            .to_string_lossy()
            .to_string()
    });

    let args = Rc::new(
        theme_path
            .map(|t| vec!["-s".to_string(), t.clone()])
            .unwrap_or(vec![]),
    );

    for module in modules {
        let (sender, receiver) = mpsc::channel::<()>(1);

        let path = CHAMELEON_BIN_ROOT.join(module.as_ref());
        ps.insert(module, sender.clone());

        let args = args.clone();
        spawn_local(async move {
            if let Err(e) = run_process_manager(path, &*args, receiver).await {
                error!("Process manager error: {}", e);
            }
        });

        let (_, config_name) = module.as_ref().split_once('-').unwrap();
        let config_name = config_name.to_string();

        let config_path =
            resolve_path(&format!("~/.config/chameleon/{}.toml", config_name))
                .unwrap();

        fs::exists(&config_path)?;

        fw = fw
            .add(
                config_path,
                Box::new(move |_| {
                    info!("{} restarted", config_name);
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        sender.send(()).await.unwrap();
                    });
                }),
            )
            .unwrap();
    }

    fw.run();

    signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn run_process_manager<I, S>(
    path: PathBuf,
    args: I,
    mut rx: mpsc::Receiver<()>,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    info!("Starting process: {:?}", path);
    let mut child =
        Some(Command::new(&path).args(args).kill_on_drop(true).spawn()?);

    loop {
        tokio::select! {
            _ = rx.recv() => {
                if let Some(c) = child.as_mut() {
                    info!("Killing existing process");
                    let _ = c.kill().await;

                    while let Some(c) = child.as_mut() {
                        match c.try_wait() {
                            Ok(Some(status)) => {
                               info!("Process exited with status: {:?}", status);
                                break;
                            }
                            Ok(None) => {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                            Err(e) => {
                                error!("Error waiting for process: {}", e);
                                break;
                            }
                        }
                    }
                }

                info!("Restart");
                child = Some(Command::new(&path).spawn()?);
            }
        }
    }
}
