#![forbid(unsafe_code)]
mod cli;
mod preset;
mod process_manager;

use crate::cli::{Args, CliCommand, StartCommand};
use crate::preset::Preset;
use crate::process_manager::ProcessManager;
use chameleon_shared::{CHAMELEON_PRESETS_ROOT, HOME, init_tracing_subscriber};
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::process::Command;

/// Default path to chameleon binaries
static CHAMELEON_BIN_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".chameleon/bin"));

#[tokio::main(flavor = "local")]
async fn main() {
    init_tracing_subscriber();

    let args: Args = argh::from_env();

    if cfg!(debug_assertions) {
        let cwd = env::current_dir().unwrap();
        let exe = env::current_exe().unwrap();
    } else {
    }

    match args.cmd {
        CliCommand::Start(cmd) => start(cmd).await,
    }
}

async fn start(cmd: StartCommand) {
    let preset_path = cmd
        .preset_root
        .as_ref()
        .unwrap_or(&*CHAMELEON_PRESETS_ROOT)
        .join(cmd.preset_name);

    let preset = Preset::load(&preset_path);
    let bin_root = cmd.bin_root.as_ref().unwrap_or(&*CHAMELEON_BIN_ROOT);
    let mut pm = ProcessManager::default();

    for module in preset.enabled {
        let exec_path = bin_root.join(module.as_ref());
        let module_process = Command::new(exec_path);

        pm.spawn(module_process).await.unwrap();
    }

    pm.run().await.unwrap();
}
