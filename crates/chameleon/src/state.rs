use crate::config::{Config, Module};
use crate::ipc::send_command;
use crate::{CHAMELEON_BIN_ROOT, ipc};
use anyhow::Result;
use chameleon_shared::utils::resolve_path;
use chameleon_shared::watcher::FilesWatcher;
use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_local;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub type PsMap = HashMap<Module, Sender<()>>;

const CONFIG_PATH: &str = "~/.config/chameleon/config.toml";

static STATE: OnceLock<RwLock<State>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct State {
    ps: PsMap,
    args: Vec<String>,
    token: CancellationToken,
}

impl State {
    pub async fn load_config_and_recreate() {
        let config = Config::load(resolve_path(CONFIG_PATH).unwrap());
        let mut state = State::default();
        state.args = config.args();

        if config.modules.is_empty() {
            warn!("No modules in config. Exit");
        }

        info!("Modules: {:?}", config.modules);
        let mut fw = FilesWatcher::new().unwrap();

        for module in config.modules.iter().map(|m| m.clone()) {
            state.handle_module(&mut fw, module).await;
        }

        fw.add2(
            resolve_path(CONFIG_PATH).unwrap(),
            Box::new(move |_| {
                send_command(ipc::Command::RestartAll);
            }),
        )
        .unwrap();

        state.token = fw.run();

        if STATE.get().is_some() {
            *State::write().await = state;
        } else {
            STATE.set(RwLock::new(state)).expect("already checked");
        }
    }

    pub async fn restart_module(module: Module) {
        if let Some(sender) = State::read().await.ps.get(&module) {
            sender.send(()).await.unwrap();
        }
    }

    async fn handle_module(&mut self, fw: &mut FilesWatcher, module: Module) {
        let (sender, receiver) = mpsc::channel::<()>(1);

        self.ps.insert(module, sender.clone());

        let token = self.token.clone();
        spawn_local(async move {
            if let Err(e) =
                Self::run_process_manager(token, module, receiver).await
            {
                error!("Process manager error: {}", e);
            }
        });

        self.start_module_watcher(fw, module, sender).await.unwrap();
    }

    async fn start_module(&self, module: Module) -> Result<Child> {
        let path = CHAMELEON_BIN_ROOT.join(module.as_ref());
        info!("Starting process: {:?}", path);

        let child = Command::new(&path)
            .args(&self.args)
            .kill_on_drop(true)
            .spawn()?;

        Ok(child)
    }

    async fn start_module_watcher(
        &mut self,
        fw: &mut FilesWatcher,
        module: Module,
        sender: Sender<()>,
    ) -> Result<()> {
        let (_, config_name) = module.as_ref().split_once('-').unwrap();
        let config_name = config_name.to_string();

        let config_path =
            resolve_path(&format!("~/.config/chameleon/{}.toml", config_name))
                .unwrap();

        fs::exists(&config_path)?;

        fw.add2(
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

        Ok(())
    }

    async fn run_process_manager(
        token: CancellationToken,
        module: Module,
        mut rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        let mut maybe_child =
            Some(State::read().await.start_module(module).await?);

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    if let Some(child) = maybe_child.as_mut() {
                        info!("Killing existing process");
                        let _ = child.kill().await;

                        while let Some(child) = maybe_child.as_mut() {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    info!("Process exited with status: {:?}", status);
                                    break;
                                }
                                Ok(None) => {
                                    tokio::time::sleep(Duration::from_millis(120)).await;
                                }
                                Err(e) => {
                                    error!("Error waiting for process: {}", e);
                                    break;
                                }
                            }
                        }
                    }

                    info!("Restart");
                    maybe_child = Some(State::read().await.start_module(module).await?);
                }
                _ = token.cancelled() => {
                    warn!("Cancelling {:?} manager process", module);

                    if let Some(mut child) = maybe_child.take() {
                        return Ok(child.kill().await?);
                    }

                    return Ok(());
                }
            }
        }
    }

    pub async fn write() -> RwLockWriteGuard<'static, State> {
        STATE.get().unwrap().write().await
    }

    pub async fn read() -> RwLockReadGuard<'static, State> {
        STATE.get().unwrap().read().await
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.token.cancel();
    }
}
