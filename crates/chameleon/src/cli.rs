use argh::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "🦎 Modular and highly customizable Wayland shell")]
pub struct Args {
    #[argh(subcommand)]
    pub cmd: CliCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CliCommand {
    Start(StartCommand),
}

#[derive(FromArgs)]
#[argh(description = "Applies preset")]
#[argh(subcommand, name = "run")]
pub struct StartCommand {
    #[argh(positional)]
    pub preset_name: String,

    #[argh(option)]
    #[argh(description = "path to folder with binaries")]
    pub bin_root: Option<PathBuf>,

    #[argh(option)]
    #[argh(description = "path to folder with presets")]
    pub preset_root: Option<PathBuf>,
}
