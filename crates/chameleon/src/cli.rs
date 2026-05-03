use argh::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "🦎 Modular and highly customizable Wayland shell")]
pub struct Args {
    #[argh(subcommand)]
    pub cmd: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Apply(ApplyCmd),
}

#[derive(FromArgs)]
#[argh(description = "Applies preset")]
#[argh(subcommand, name = "run")]
pub struct ApplyCmd {
    #[argh(positional)]
    pub preset_name: String,

    #[argh(option)]
    #[argh(description = "path to folder with chameleon binaries")]
    pub bin_root: Option<PathBuf>,

    #[argh(option)]
    #[argh(description = "path to folder with presets")]
    pub presets_root: Option<PathBuf>,
}
