#![forbid(unsafe_code)]
mod cli;
mod config;
mod ipc;
mod state;

use crate::cli::{Args, CliCommand};
use crate::ipc::{send_command_with_exit, wait_command};
use crate::state::State;
use anyhow::Result;
use chameleon_shared::{HOME, init_tracing_subscriber};
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::signal;
use tokio::task::spawn_local;

/// Default path to chameleon binaries
static CHAMELEON_BIN_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".chameleon/bin"));

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    init_tracing_subscriber();
    run(argh::from_env()).await?;
    signal::ctrl_c().await?;

    Ok(())
}

async fn run(args: Args) -> Result<()> {
    match args.cmd {
        None => {
            spawn_local(wait_command());
            State::load_config_and_recreate().await
        }
        Some(cmd) => match cmd {
            CliCommand::HealthCheck(_) => {
                send_command_with_exit(ipc::Command::Health)
            }
        },
    }

    Ok(())
}
