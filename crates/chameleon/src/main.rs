#![forbid(unsafe_code)]
mod cli;
mod preset;
use chameleon_shared::init_tracing_subscriber;
use std::fs;
use tracing::{error, warn};

use crate::cli::{Args, Command};

#[tokio::main(flavor = "local")]
async fn main() {
    init_tracing_subscriber();

    let args: Args = argh::from_env();

    match args.cmd {
        Command::Apply(run_cmd) => {
            let str = match fs::read_to_string(&config_path) {
                Ok(str) => str,
                Err(e) => {
                    warn!(
                        "Failed to open config file at {config_path:?}. {e}. The default configuration will be used."
                    );
                    Config::default()
                }
            };

            let preset = match toml::from_str(&input) {
                Ok(config) => config,
                Err(e) => {
                    error!("{e}");
                    std::process::exit(-1);
                }
            };
        }
    }
}
