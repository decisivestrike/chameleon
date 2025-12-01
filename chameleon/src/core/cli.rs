use std::{env::home_dir, path::PathBuf};

fn default_config_path() -> PathBuf {
    home_dir().unwrap().join(".config/chameleon/config.toml")
}

fn default_styles_path() -> PathBuf {
    home_dir().unwrap().join(".config/chameleon/styles.css")
}

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "🦎 Highly customizable Wayland shell")]
pub struct Args {
    #[argh(
        option,
        short = 'c',
        default = "default_config_path()",
        description = "config path"
    )]
    pub config_path: PathBuf,

    #[argh(
        option,
        short = 's',
        default = "default_styles_path()",
        description = "styles path"
    )]
    pub style_path: PathBuf,
}
