use argh::FromArgs;
use chameleon_shared::CHAMELEON_CONFIG_ROOT;
use chameleon_shared::utils::resolve_path;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "Panel for Chameleon")]
pub struct Args {
    #[argh(
        option,
        short = 'c',
        description = "config path",
        default = "default_config_path()",
        from_str_fn(resolve_path)
    )]
    pub config_path: PathBuf,

    #[argh(
        option,
        short = 's',
        description = "styles path",
        default = "default_styles_path()",
        from_str_fn(resolve_path)
    )]
    pub styles_path: PathBuf,
}

fn default_config_path() -> PathBuf {
    CHAMELEON_CONFIG_ROOT.join("panel.toml")
}

fn default_styles_path() -> PathBuf {
    CHAMELEON_CONFIG_ROOT.join("styles.css")
}
