use argh::FromArgs;
use std::{env::home_dir, path::PathBuf, sync::LazyLock};

pub static ARGS: LazyLock<Args> = LazyLock::new(|| argh::from_env());

#[derive(FromArgs)]
#[argh(description = "🦎 Highly customizable Wayland shell")]
pub struct Args {
    #[argh(
        option,
        short = 'c',
        default = "Args::default_config_path()",
        description = "config path"
    )]
    pub config_path: PathBuf,

    #[argh(
        option,
        short = 's',
        default = "Args::default_styles_path()",
        description = "styles path"
    )]
    pub styles_path: PathBuf,

    #[argh(switch, short = 'w', description = "watch mode for styles")]
    pub watch_enabled: bool,
}

impl Args {
    fn default_config_path() -> PathBuf {
        home_dir().unwrap().join(".config/chameleon/config.toml")
    }

    fn default_styles_path() -> PathBuf {
        home_dir().unwrap().join(".config/chameleon/styles.css")
    }
}
