pub mod errors;

use serde::de::DeserializeOwned;
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fmt, fs};

pub static DEFAULT_CONFIG_FOLDER: LazyLock<PathBuf> = LazyLock::new(|| {
    let maybe_home = home_dir();

    if let Some(home) = maybe_home {
        home.join(".config").join("chameleon")
    } else {
        panic!("Cant get $HOME")
    }
});

pub fn read_config<Config, P>(config_path: P) -> Config
where
    Config: DeserializeOwned,
    P: AsRef<Path> + fmt::Debug,
{
    let toml_str = match fs::read_to_string(&config_path) {
        Ok(file) => file,
        Err(e) => {
            log::error!("Failed to open config file at {config_path:?}. {e}");
            std::process::exit(-1);
        }
    };

    match toml::from_str(&toml_str) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            std::process::exit(-1);
        }
    }
}
