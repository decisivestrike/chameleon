use argh::FromArgs;
use resolve_path::PathResolveExt;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "🦎 Highly customizable Wayland shell")]
pub struct Command {
    #[argh(
        option,
        short = 'c',
        default = "default_config_folder()",
        description = "config folder path",
        from_str_fn(parse_path)
    )]
    pub config_folder: PathBuf,

    #[argh(switch, short = 'w', description = "watch mode for styles")]
    pub watch_enabled: bool,

    #[argh(subcommand)]
    pub module: ModuleCommand,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub enum ModuleCommand {
    Launcher(LauncherCommand),
}

/// Command for launcher
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "launcher")]
pub struct LauncherCommand {
    #[argh(positional)]
    action: String,

    #[argh(switch, short = 't', description = "toggle launcher visibility")]
    pub toggle_launcher: bool,
}

fn parse_path(value: &str) -> Result<PathBuf, String> {
    Ok(value
        .try_resolve()
        .map_err(|e| e.to_string())?
        .to_path_buf())
}

fn default_config_folder() -> PathBuf {
    parse_path("~/.config/chameleon").expect("")
}
